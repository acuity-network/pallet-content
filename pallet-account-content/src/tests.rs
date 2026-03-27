use crate::{mock::*, AccountItemIdIndex, AccountItemIds, Error, Event, Pallet as AccountContent};
use pallet_content::{IpfsHash, Item, ItemId, Nonce, Pallet as Content};
use polkadot_sdk::frame_support::{assert_noop, assert_ok};

const REVISIONABLE: u8 = 1 << 0;

fn publish_item(owner: u64, nonce: Nonce) -> ItemId {
    assert_ok!(Content::<Test>::publish_item(
        RuntimeOrigin::signed(owner),
        nonce,
        Default::default(),
        REVISIONABLE,
        Default::default(),
        Default::default(),
        IpfsHash::default(),
    ));

    System::events()
        .into_iter()
        .rev()
        .find_map(|record| match record.event {
            RuntimeEvent::Content(pallet_content::Event::PublishItem { item_id, .. }) => {
                Some(item_id)
            }
            _ => None,
        })
        .expect("publish item event must exist")
}

fn insert_owned_item(owner: u64, fill: u8) -> ItemId {
    let item_id = ItemId([fill; 32]);
    pallet_content::ItemState::<Test>::insert(
        item_id.clone(),
        Item {
            owner,
            revision_id: 0,
            flags: REVISIONABLE,
        },
    );
    item_id
}

#[test]
fn add_item_records_membership_and_event() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());

        assert_ok!(AccountContent::<Test>::add_item(
            RuntimeOrigin::signed(1),
            item_id.clone()
        ));

        assert_eq!(
            AccountContent::<Test>::get_item_exists(1, item_id.clone()),
            true
        );
        assert_eq!(AccountContent::<Test>::get_item_count(1), 1);
        assert_eq!(
            AccountContent::<Test>::get_all_items(1).into_inner(),
            vec![item_id.clone()]
        );
        assert_eq!(AccountItemIdIndex::<Test>::get(1, item_id.clone()), 1);
        assert_eq!(
            AccountItemIds::<Test>::get(1).into_inner(),
            vec![item_id.clone()]
        );

        System::assert_has_event(
            Event::<Test>::AddItem {
                account: 1,
                item_id,
            }
            .into(),
        );
    });
}

#[test]
fn add_item_rejects_duplicate_membership() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());

        assert_ok!(AccountContent::<Test>::add_item(
            RuntimeOrigin::signed(1),
            item_id.clone()
        ));
        assert_noop!(
            AccountContent::<Test>::add_item(RuntimeOrigin::signed(1), item_id),
            Error::<Test>::ItemAlreadyAdded
        );
    });
}

#[test]
fn add_item_rejects_missing_or_unowned_content() {
    new_test_ext().execute_with(|| {
        let missing_item_id = ItemId([7; 32]);

        assert_noop!(
            AccountContent::<Test>::add_item(RuntimeOrigin::signed(1), missing_item_id.clone()),
            Error::<Test>::ItemNotFound
        );

        let item_id = insert_owned_item(1, 1);
        assert_noop!(
            AccountContent::<Test>::add_item(RuntimeOrigin::signed(2), item_id),
            Error::<Test>::WrongAccount
        );
    });
}

#[test]
fn add_item_rejects_full_account_list() {
    new_test_ext().execute_with(|| {
        let item_id_1 = insert_owned_item(1, 1);
        let item_id_2 = insert_owned_item(1, 2);
        let item_id_3 = insert_owned_item(1, 3);

        assert_ok!(AccountContent::<Test>::add_item(
            RuntimeOrigin::signed(1),
            item_id_1
        ));
        assert_ok!(AccountContent::<Test>::add_item(
            RuntimeOrigin::signed(1),
            item_id_2
        ));
        assert_noop!(
            AccountContent::<Test>::add_item(RuntimeOrigin::signed(1), item_id_3),
            Error::<Test>::AccountItemsFull
        );
    });
}

#[test]
fn remove_item_swaps_last_item_and_updates_indexes() {
    new_test_ext().execute_with(|| {
        let item_id_1 = insert_owned_item(1, 1);
        let item_id_2 = insert_owned_item(1, 2);

        assert_ok!(AccountContent::<Test>::add_item(
            RuntimeOrigin::signed(1),
            item_id_1.clone()
        ));
        assert_ok!(AccountContent::<Test>::add_item(
            RuntimeOrigin::signed(1),
            item_id_2.clone()
        ));

        assert_ok!(AccountContent::<Test>::remove_item(
            RuntimeOrigin::signed(1),
            item_id_1.clone()
        ));

        assert_eq!(
            AccountContent::<Test>::get_item_exists(1, item_id_1.clone()),
            false
        );
        assert_eq!(
            AccountContent::<Test>::get_item_exists_by_account(1, item_id_2.clone()),
            true
        );
        assert_eq!(AccountContent::<Test>::get_item_count_by_account(1), 1);
        assert_eq!(
            AccountContent::<Test>::get_all_items_by_account(1).into_inner(),
            vec![item_id_2.clone()]
        );
        assert_eq!(AccountItemIdIndex::<Test>::get(1, item_id_1.clone()), 0);
        assert_eq!(AccountItemIdIndex::<Test>::get(1, item_id_2.clone()), 1);

        System::assert_has_event(
            Event::<Test>::RemoveItem {
                account: 1,
                item_id: item_id_1,
            }
            .into(),
        );
    });
}

#[test]
fn remove_item_rejects_missing_membership() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());

        assert_noop!(
            AccountContent::<Test>::remove_item(RuntimeOrigin::signed(1), item_id),
            Error::<Test>::ItemNotAdded
        );
    });
}

#[test]
fn remove_item_rejects_stale_non_owner_membership() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());
        assert_ok!(AccountContent::<Test>::add_item(
            RuntimeOrigin::signed(1),
            item_id.clone()
        ));

        pallet_content::ItemState::<Test>::insert(
            item_id.clone(),
            Item {
                owner: 2,
                revision_id: 0,
                flags: REVISIONABLE,
            },
        );

        assert_noop!(
            AccountContent::<Test>::remove_item(RuntimeOrigin::signed(1), item_id),
            Error::<Test>::WrongAccount
        );
    });
}
