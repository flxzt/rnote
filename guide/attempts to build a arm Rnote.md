# Existing attempts to build rnote on windows arm64:

Q:
Have you tried building it on your machine? The best way to get going is that someone with interest in a specific platform who actually owns a device running said platform starts experimenting. You could start by setting up msys2 and checking if all dependencies are available. See this workflow for useful pointers: https://github.com/flxzt/rnote/blob/main/.github/workflows/release-windows.yml

A:
By using MSYS2 clangarm64, all dependencies except diffutils and gcc are available. Besides, emulated x86_64 diffutils is available too. The rust toolchain for clang-aarch64 is provided by MSYS2 project. By changing compiler to clang, the application can be built. However, two problems still exists:

Although the executable itself can be built, the installer can not be built correctly without modification. The main reason is that the package script use mingw64 as prefix by default but using the toolchain explained above will install all dependencies to {msys2_dir}/clangarm64 and thus dlls are not recoignized by the inno-build.py script and iscc. I believe by changing patterns it is not that hard to solve :)
Another problem seems more serious. When I try to run the compiled rnote, even in the msys2 environment where all the dependencies can be found normally, the executable is not able to run correctly. It do not show the gui and print "thread 'main' has overflowed its stack".
I wonder if there is any advice.