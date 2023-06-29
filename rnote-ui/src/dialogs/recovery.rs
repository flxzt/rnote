use cairo::glib::{self, clone, Cast, StaticType};
use gtk4::{ConstantExpression, ListItem, PropertyExpression, SignalListItemFactory};

use crate::{appwindow::RnAppWindow, recoveryrow::RnRecoveryRow};

pub(crate) async fn dialog_recover_documents(appwindow: &RnAppWindow) {
    setup_recovery_rows(appwindow);
}

fn setup_recovery_rows(appwindow: &RnAppWindow) {
    let primary_list_factory = SignalListItemFactory::new();
    primary_list_factory.connect_setup(clone!(@weak appwindow => move |_, list_item| {
        let list_item = list_item.downcast_ref::<ListItem>().unwrap();

        let recoveryrow = RnRecoveryRow::new();
        recoveryrow.init(&appwindow);
        list_item.set_child(Some(&recoveryrow));

        let list_item_expr = ConstantExpression::new(list_item);
        let recoveryinfo_expr =
            PropertyExpression::new(ListItem::static_type(), Some(&list_item_expr), "item");

        // recoveryrow.
        // recoveryinfo_expr.chain_closure(|| ())

    }));
}
