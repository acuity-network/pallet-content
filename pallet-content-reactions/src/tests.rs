use crate::{mock::*, Emoji, Error, Event, ItemAccountReactions, Pallet as ContentReactions};
use pallet_content::{IpfsHash, Item, ItemId, Nonce, Pallet as Content};
use polkadot_sdk::frame_support::{assert_noop, assert_ok};

const REVISIONABLE: u8 = 1 << 0;
const GRINNING_FACE: Emoji = Emoji(0x1F600);
const SMILING_FACE_WITH_HEART_EYES: Emoji = Emoji(0x1F60D);
const PARTY_POPPER: Emoji = Emoji(0x1F389);

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
fn add_reaction_records_storage_and_event() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());

        assert_ok!(ContentReactions::<Test>::add_reaction(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            GRINNING_FACE,
        ));

        assert_eq!(
            ItemAccountReactions::<Test>::get(item_id.clone(), 2)
                .expect("reaction entry must exist")
                .into_inner(),
            vec![GRINNING_FACE]
        );

        System::assert_has_event(
            Event::<Test>::AddReaction {
                item_id,
                item_owner: 1,
                reactor: 2,
                emoji: GRINNING_FACE,
            }
            .into(),
        );
    });
}

#[test]
fn add_reaction_is_noop_for_duplicate_emoji() {
    new_test_ext().execute_with(|| {
        let item_id = publish_item(1, Nonce::default());

        assert_ok!(ContentReactions::<Test>::add_reaction(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            GRINNING_FACE,
        ));
        assert_ok!(ContentReactions::<Test>::add_reaction(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            GRINNING_FACE,
        ));

        assert_eq!(
            ItemAccountReactions::<Test>::get(item_id, 2)
                .expect("reaction entry must exist")
                .into_inner(),
            vec![GRINNING_FACE]
        );

        let add_events = System::events()
            .into_iter()
            .filter(|record| {
                matches!(
                    record.event,
                    RuntimeEvent::ContentReactions(Event::AddReaction { .. })
                )
            })
            .count();

        assert_eq!(add_events, 1);
    });
}

#[test]
fn add_reaction_rejects_missing_item() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ContentReactions::<Test>::add_reaction(
                RuntimeOrigin::signed(1),
                ItemId([7; 32]),
                GRINNING_FACE,
            ),
            Error::<Test>::ItemNotFound
        );
    });
}

#[test]
fn add_reaction_rejects_invalid_emoji_values() {
    new_test_ext().execute_with(|| {
        let item_id = insert_owned_item(1, 1);

        assert_noop!(
            ContentReactions::<Test>::add_reaction(
                RuntimeOrigin::signed(2),
                item_id.clone(),
                Emoji(0)
            ),
            Error::<Test>::InvalidEmoji
        );
        assert_noop!(
            ContentReactions::<Test>::add_reaction(
                RuntimeOrigin::signed(2),
                item_id.clone(),
                Emoji(0xD800),
            ),
            Error::<Test>::InvalidEmoji
        );
        assert_noop!(
            ContentReactions::<Test>::add_reaction(
                RuntimeOrigin::signed(2),
                item_id,
                Emoji(0x110000),
            ),
            Error::<Test>::InvalidEmoji
        );
    });
}

#[test]
fn add_reaction_rejects_when_max_emojis_reached() {
    new_test_ext().execute_with(|| {
        let item_id = insert_owned_item(1, 1);

        assert_ok!(ContentReactions::<Test>::add_reaction(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            GRINNING_FACE,
        ));
        assert_ok!(ContentReactions::<Test>::add_reaction(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            SMILING_FACE_WITH_HEART_EYES,
        ));
        assert_noop!(
            ContentReactions::<Test>::add_reaction(
                RuntimeOrigin::signed(2),
                item_id.clone(),
                PARTY_POPPER,
            ),
            Error::<Test>::TooManyEmojis
        );

        assert_eq!(
            ItemAccountReactions::<Test>::get(item_id, 2)
                .expect("reaction entry must exist")
                .into_inner(),
            vec![GRINNING_FACE, SMILING_FACE_WITH_HEART_EYES]
        );
    });
}

#[test]
fn remove_reaction_updates_storage_and_event() {
    new_test_ext().execute_with(|| {
        let item_id = insert_owned_item(1, 1);

        assert_ok!(ContentReactions::<Test>::add_reaction(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            GRINNING_FACE,
        ));
        assert_ok!(ContentReactions::<Test>::add_reaction(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            SMILING_FACE_WITH_HEART_EYES,
        ));

        assert_ok!(ContentReactions::<Test>::remove_reaction(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            GRINNING_FACE,
        ));

        assert_eq!(
            ItemAccountReactions::<Test>::get(item_id.clone(), 2)
                .expect("reaction entry must exist")
                .into_inner(),
            vec![SMILING_FACE_WITH_HEART_EYES]
        );

        System::assert_has_event(
            Event::<Test>::RemoveReaction {
                item_id,
                item_owner: 1,
                reactor: 2,
                emoji: GRINNING_FACE,
            }
            .into(),
        );
    });
}

#[test]
fn remove_reaction_is_noop_when_emoji_missing() {
    new_test_ext().execute_with(|| {
        let item_id = insert_owned_item(1, 1);

        assert_ok!(ContentReactions::<Test>::add_reaction(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            GRINNING_FACE,
        ));
        assert_ok!(ContentReactions::<Test>::remove_reaction(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            PARTY_POPPER,
        ));

        assert_eq!(
            ItemAccountReactions::<Test>::get(item_id, 2)
                .expect("reaction entry must exist")
                .into_inner(),
            vec![GRINNING_FACE]
        );

        let remove_events = System::events()
            .into_iter()
            .filter(|record| {
                matches!(
                    record.event,
                    RuntimeEvent::ContentReactions(Event::RemoveReaction { .. })
                )
            })
            .count();

        assert_eq!(remove_events, 0);
    });
}

#[test]
fn remove_reaction_clears_empty_storage_entry() {
    new_test_ext().execute_with(|| {
        let item_id = insert_owned_item(1, 1);

        assert_ok!(ContentReactions::<Test>::add_reaction(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            GRINNING_FACE,
        ));
        assert_ok!(ContentReactions::<Test>::remove_reaction(
            RuntimeOrigin::signed(2),
            item_id.clone(),
            GRINNING_FACE,
        ));

        assert_eq!(ItemAccountReactions::<Test>::get(item_id, 2), None);
    });
}
