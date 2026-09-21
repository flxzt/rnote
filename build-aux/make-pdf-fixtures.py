#!/usr/bin/env python3
"""Generate Pdf documents to measure Pdf import with.

`crates/rnote-engine/examples/pdf_import_bench.rs` reports how much importing a Pdf allocates,
which depends heavily on the document: a Pdf of vector text barely allocates anything per page,
while a scanned page carries an image that has to be decoded and re-encoded. These fixtures cover
that range, so an import change can be checked against all of it instead of one lucky document:

  book120.pdf   120 pages of vector text and rects (through ghostscript)     ~0.04 MB
                -> nothing is rasterized in the Pdf, so vector import is cheap
  scan60.pdf     60 pages of structured "sensor" noise, Jpeg RGB             ~35 MB
                -> phone-scan-like content, the common real-world case
  noise30.pdf    30 pages of uniform random noise, Jpeg gray                 ~51 MB
                -> incompressible in the Pdf filter and in raster form

Every page is 1011 x 1430 pt. The content comes from fixed seeds, so the same pixels (and the same
benchmark hashes) come out on every run: the two image documents are byte-identical, and the
vector one only differs in the metadata ghostscript stamps into it.

Requirements: python3 with numpy and pillow, plus ghostscript (`gs`) on PATH.

Usage: python3 build-aux/make-pdf-fixtures.py [OUTDIR]     (default: /tmp/rnote-fixtures)

Then, for example:

  cargo run --release -p rnote-engine --example pdf_import_bench -- /tmp/rnote-fixtures/scan60.pdf bitmap 60
"""

import os
import subprocess
import sys
from io import BytesIO

import numpy as np
from PIL import Image

PAGE_W, PAGE_H = 1011, 1430


# --------------------------------------------------------------------------------------
# Minimal Pdf writer: one full-page image XObject per page. Enough for the fixtures, and
# avoids pulling in a Pdf library just to measure import cost.
# --------------------------------------------------------------------------------------
class Pdf:
    def __init__(self) -> None:
        # object number 1 is the catalog, 2 the page tree, filled in flush()
        self.objects: list[bytes] = []

    def add(self, body: bytes) -> int:
        self.objects.append(body)
        return len(self.objects) + 2  # +1 for the two objects prepended in flush()

    def flush(self, path: str, pages: list[tuple[int, int]]) -> None:
        """pages: list of (image_object_number, content_object_number)"""
        head = [
            b"<< /Type /Catalog /Pages 2 0 R >>",
            b"<< /Type /Pages /Kids ["
            + b" ".join(f"{n} 0 R".encode() for n, _ in pages)
            + f"] /Count {len(pages)} >>".encode(),
        ]
        body = head + self.objects

        out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
        offsets = [0]
        for i, obj in enumerate(body, start=1):
            offsets.append(len(out))
            out += f"{i} 0 obj\n".encode() + obj + b"\nendobj\n"
        xref_pos = len(out)
        out += f"xref\n0 {len(body) + 1}\n".encode()
        out += b"0000000000 65535 f \n"
        for off in offsets[1:]:
            out += f"{off:010d} 00000 n \n".encode()
        out += (
            f"trailer\n<< /Size {len(body) + 1} /Root 1 0 R >>\nstartxref\n{xref_pos}\n%%EOF\n"
        ).encode()
        with open(path, "wb") as f:
            f.write(out)


def image_xobject(jpeg: bytes, w: int, h: int, gray: bool) -> bytes:
    cs = b"/ColorSpace /DeviceGray" if gray else b"/ColorSpace /DeviceRGB"
    return (
        b"<< /Type /XObject /Subtype /Image "
        + f"/Width {w} /Height {h} ".encode()
        + cs
        + b" /BitsPerComponent 8 /Filter /DCTDecode "
        + f"/Length {len(jpeg)} >>\nstream\n".encode()
        + jpeg
        + b"\nendstream"
    )


def content_stream(w: int, h: int, img_obj: int) -> bytes:
    ops = f"q {w} 0 0 {h} 0 0 cm /Im0 Do Q".encode()
    return b"<< /Length " + str(len(ops)).encode() + b" >>\nstream\n" + ops + b"\nendstream"


def jpeg_of(img: Image.Image, quality: int) -> bytes:
    buf = BytesIO()
    img.save(buf, format="JPEG", quality=quality, subsampling=0, optimize=False)
    return buf.getvalue()


def write_image_pdf(path: str, pages: list[Image.Image], quality: int) -> None:
    pdf = Pdf()
    page_refs: list[tuple[int, int]] = []
    for img in pages:
        gray = img.mode == "L"
        jpeg = jpeg_of(img, quality)
        img_no = pdf.add(image_xobject(jpeg, img.width, img.height, gray))
        content_no = pdf.add(content_stream(img.width, img.height, img_no))
        page_no = pdf.add(
            b"<< /Type /Page /Parent 2 0 R "
            + f"/MediaBox [0 0 {img.width} {img.height}] ".encode()
            + b"/Resources << /XObject << /Im0 "
            + f"{img_no} 0 R >> >> ".encode()
            + b"/Contents "
            + f"{content_no} 0 R >>".encode()
        )
        page_refs.append((page_no, content_no))
    pdf.flush(path, page_refs)


# --------------------------------------------------------------------------------------
# Fixture content
# --------------------------------------------------------------------------------------
def scan_pages(n: int) -> list[Image.Image]:
    """Phone-scan-like: smooth uneven illumination, paper grain, faint printed text rows."""
    yy, xx = np.mgrid[0:PAGE_H, 0:PAGE_W].astype(np.float32)
    out = []
    for i in range(n):
        rng = np.random.default_rng(1000 + i)
        # uneven illumination: a smooth 2d gradient plus a soft shadow in one corner
        grad = 235.0 - 40.0 * (xx / PAGE_W) - 25.0 * (yy / PAGE_H)
        shadow = 55.0 * np.exp(
            -(
                ((xx - PAGE_W * 0.85) ** 2 + (yy - PAGE_H * 0.2) ** 2)
                / (2 * (PAGE_W * 0.35) ** 2)
            )
        )
        base = grad - shadow
        # sensor grain
        base += rng.normal(0.0, 16.0, size=base.shape).astype(np.float32)
        # text-like horizontal strokes, 40 rows
        for row in range(40):
            y0 = 90 + row * 32 + int(rng.integers(-2, 3))
            x0 = 90 + int(rng.integers(0, 60))
            x1 = x0 + int(rng.integers(400, 800))
            base[y0 : y0 + int(rng.integers(5, 11)), x0:x1] -= float(rng.integers(90, 150))
        # slight vertical streaking, as scanners produce
        streak = rng.normal(0.0, 4.0, size=(1, PAGE_W)).astype(np.float32)
        base += streak
        rgb = np.stack([base * 1.00, base * 0.985, base * 0.955], axis=-1)
        out.append(Image.fromarray(np.clip(rgb, 0, 255).astype(np.uint8), "RGB"))
    return out


def noise_pages(n: int) -> list[Image.Image]:
    """Uniform random noise: incompressible in both the Pdf filter and in raster form."""
    out = []
    for i in range(n):
        rng = np.random.default_rng(5000 + i)
        arr = rng.integers(0, 256, size=(PAGE_H, PAGE_W), dtype=np.uint8)
        out.append(Image.fromarray(arr, "L"))
    return out


def book_pdf(path: str, n: int) -> None:
    """Vector text + rects through ghostscript, so nothing is rasterized in the Pdf."""
    ps = ["%!PS-Adobe-3.0", f"%%Pages: {n}"]
    for i in range(n):
        ps.append("/Helvetica-Bold findfont 42 scalefont setfont")
        ps.append(f"72 1330 moveto (Chapter {i + 1} - Rnote memory fixtures) show")
        ps.append("/Helvetica findfont 20 scalefont setfont")
        for line in range(28):
            ps.append(
                f"72 {1270 - line * 40} moveto (page {i + 1}, line {line + 1}: "
                f"the quick brown fox jumps over the lazy dog {i * 31 + line * 7}) show"
            )
        ps.append("1 0.4 0.2 setrgbcolor")
        for r in range(6):
            x = 560 + (r % 3) * 130
            y = 1050 - (r // 3) * 160
            ps.append(f"newpath {x} {y} 55 0 360 arc fill")
        ps.append("0.1 0.3 0.7 setrgbcolor")
        ps.append("72 60 moveto 900 30 rlineto 0 26 rlineto -900 30 rlineto closepath fill")
        ps.append("showpage")
    ps.append("%%EOF")
    src = path.replace(".pdf", ".ps")
    with open(src, "w") as f:
        f.write("\n".join(ps))
    subprocess.run(
        [
            "gs",
            "-q",
            "-dNOPAUSE",
            "-dBATCH",
            "-sDEVICE=pdfwrite",
            f"-g{PAGE_W}x{PAGE_H}",
            "-dFIXEDMEDIA",
            "-dPDFFitPage",
            f"-sOutputFile={path}",
            src,
        ],
        check=True,
    )
    os.unlink(src)


def main() -> None:
    outdir = sys.argv[1] if len(sys.argv) > 1 else "/tmp/rnote-fixtures"
    os.makedirs(outdir, exist_ok=True)

    book = os.path.join(outdir, "book120.pdf")
    book_pdf(book, 120)
    print(f"{book:32s} {os.path.getsize(book) / 1e6:7.2f} MB")

    scan = os.path.join(outdir, "scan60.pdf")
    write_image_pdf(scan, scan_pages(60), quality=85)
    print(f"{scan:32s} {os.path.getsize(scan) / 1e6:7.2f} MB")

    noise = os.path.join(outdir, "noise30.pdf")
    write_image_pdf(noise, noise_pages(30), quality=97)
    print(f"{noise:32s} {os.path.getsize(noise) / 1e6:7.2f} MB")


if __name__ == "__main__":
    main()
