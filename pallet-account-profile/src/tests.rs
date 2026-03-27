use crate::{mock::*, AccountProfile, Error, Event, Pallet as AccountProfilePallet};
use pallet_content::{IpfsHash, Item, ItemId, Nonce, Pallet as Content, RETRACTED};
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
fn set_profile_records_membership_and_event() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());

        assert_ok!(AccountProfilePallet::<Test>::set_profile(
            RuntimeOrigin::signed(1),
            item_id.clone()
        ));

        assert_eq!(AccountProfile::<Test>::get(1), Some(item_id.clone()));
        assert_eq!(
            AccountProfilePallet::<Test>::account_profile(1),
            Some(item_id.clone())
        );

        System::assert_has_event(
            Event::<Test>::ProfileSet {
                account: 1,
                item_id,
            }
            .into(),
        );
    });
}

#[test]
fn set_profile_rejects_missing_or_unowned_content() {
    new_test_ext().execute_with(|| {
        let missing_item_id = ItemId([7; 32]);

        assert_noop!(
            AccountProfilePallet::<Test>::set_profile(
                RuntimeOrigin::signed(1),
                missing_item_id.clone()
            ),
            Error::<Test>::ItemNotFound
        );

        let item_id = insert_owned_item(1, 1);
        assert_noop!(
            AccountProfilePallet::<Test>::set_profile(RuntimeOrigin::signed(2), item_id),
            Error::<Test>::WrongAccount
        );
    });
}

#[test]
fn set_profile_rejects_retracted_content() {
    new_test_ext().execute_with(|| {
        let item_id = ItemId([9; 32]);
        pallet_content::ItemState::<Test>::insert(
            item_id.clone(),
            Item {
                owner: 1,
                revision_id: 0,
                flags: RETRACTED,
            },
        );

        assert_noop!(
            AccountProfilePallet::<Test>::set_profile(RuntimeOrigin::signed(1), item_id),
            Error::<Test>::ItemRetracted
        );
    });
}

#[test]
fn set_profile_overwrites_previous_profile() {
    new_test_ext().execute_with(|| {
        let item_id_1 = insert_owned_item(1, 1);
        let item_id_2 = insert_owned_item(1, 2);

        assert_ok!(AccountProfilePallet::<Test>::set_profile(
            RuntimeOrigin::signed(1),
            item_id_1
        ));
        assert_ok!(AccountProfilePallet::<Test>::set_profile(
            RuntimeOrigin::signed(1),
            item_id_2.clone()
        ));

        assert_eq!(AccountProfile::<Test>::get(1), Some(item_id_2.clone()));

        System::assert_has_event(
            Event::<Test>::ProfileSet {
                account: 1,
                item_id: item_id_2,
            }
            .into(),
        );
    });
}

#[test]
fn set_profile_allows_setting_same_item_again() {
    new_test_ext().execute_with(|| {
        let item_id = insert_owned_item(1, 1);

        assert_ok!(AccountProfilePallet::<Test>::set_profile(
            RuntimeOrigin::signed(1),
            item_id.clone()
        ));
        assert_ok!(AccountProfilePallet::<Test>::set_profile(
            RuntimeOrigin::signed(1),
            item_id.clone()
        ));

        assert_eq!(AccountProfile::<Test>::get(1), Some(item_id.clone()));

        let profile_set_events = System::events()
            .into_iter()
            .filter(|record| {
                matches!(
                    record.event,
                    RuntimeEvent::AccountProfile(Event::ProfileSet { .. })
                )
            })
            .count();

        assert_eq!(profile_set_events, 2);
    });
}
