use crate::{
  objects::community::ApubCommunity,
  protocol::{
    activities::{
      block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
      community::{
        announce::{AnnounceActivity, RawAnnouncableActivities},
        collection_add::CollectionAdd,
        collection_remove::CollectionRemove,
        lock_page::{LockPage, UndoLockPage},
        report::Report,
        update::UpdateCommunity,
      },
      create_or_update::{
        chat_message::CreateOrUpdateChatMessage,
        note::CreateOrUpdateNote,
        page::CreateOrUpdatePage,
      },
      deletion::{delete::Delete, delete_user::DeleteUser, undo_delete::UndoDelete},
      following::{accept::AcceptFollow, follow::Follow, undo_follow::UndoFollow},
      voting::{undo_vote::UndoVote, vote::Vote},
    },
    objects::page::Page,
    InCommunity,
  },
};
use activitypub_federation::{
  config::Data,
  protocol::context::WithContext,
  traits::ActivityHandler,
};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum SharedInboxActivities {
  PersonInboxActivities(Box<WithContext<PersonInboxActivities>>),
  GroupInboxActivities(Box<WithContext<GroupInboxActivities>>),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum GroupInboxActivities {
  Follow(Follow),
  UndoFollow(UndoFollow),
  Report(Report),
  // This is a catch-all and needs to be last
  AnnouncableActivities(RawAnnouncableActivities),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonInboxActivities {
  Follow(Follow),
  AcceptFollow(AcceptFollow),
  UndoFollow(UndoFollow),
  CreateOrUpdatePrivateMessage(CreateOrUpdateChatMessage),
  Delete(Delete),
  UndoDelete(UndoDelete),
  AnnounceActivity(AnnounceActivity),
}

/// This is necessary for user inbox, which can also receive some "announcable" activities,
/// eg a comment mention. This needs to be a separate enum so that announcables received in shared
/// inbox can fall through to be parsed as GroupInboxActivities::AnnouncableActivities.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonInboxActivitiesWithAnnouncable {
  PersonInboxActivities(Box<PersonInboxActivities>),
  AnnouncableActivities(Box<AnnouncableActivities>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum AnnouncableActivities {
  CreateOrUpdateComment(CreateOrUpdateNote),
  CreateOrUpdatePost(CreateOrUpdatePage),
  Vote(Vote),
  UndoVote(UndoVote),
  Delete(Delete),
  UndoDelete(UndoDelete),
  UpdateCommunity(UpdateCommunity),
  BlockUser(BlockUser),
  UndoBlockUser(UndoBlockUser),
  CollectionAdd(CollectionAdd),
  CollectionRemove(CollectionRemove),
  LockPost(LockPage),
  UndoLockPost(UndoLockPage),
  // For compatibility with Pleroma/Mastodon (send only)
  Page(Page),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
#[allow(clippy::enum_variant_names)]
pub enum SiteInboxActivities {
  BlockUser(BlockUser),
  UndoBlockUser(UndoBlockUser),
  DeleteUser(DeleteUser),
}

#[async_trait::async_trait]
impl InCommunity for AnnouncableActivities {
  #[tracing::instrument(skip(self, context))]
  async fn community(&self, context: &Data<LemmyContext>) -> Result<ApubCommunity, LemmyError> {
    use AnnouncableActivities::*;
    match self {
      CreateOrUpdateComment(a) => a.community(context).await,
      CreateOrUpdatePost(a) => a.community(context).await,
      Vote(a) => a.community(context).await,
      UndoVote(a) => a.community(context).await,
      Delete(a) => a.community(context).await,
      UndoDelete(a) => a.community(context).await,
      UpdateCommunity(a) => a.community(context).await,
      BlockUser(a) => a.community(context).await,
      UndoBlockUser(a) => a.community(context).await,
      CollectionAdd(a) => a.community(context).await,
      CollectionRemove(a) => a.community(context).await,
      LockPost(a) => a.community(context).await,
      UndoLockPost(a) => a.community(context).await,
      Page(_) => unimplemented!(),
    }
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{
    activity_lists::{
      GroupInboxActivities,
      PersonInboxActivities,
      PersonInboxActivitiesWithAnnouncable,
      SiteInboxActivities,
    },
    protocol::tests::{test_json, test_parse_lemmy_item},
  };

  #[test]
  fn test_group_inbox() {
    test_parse_lemmy_item::<GroupInboxActivities>("assets/lemmy/activities/following/follow.json")
      .unwrap();
    test_parse_lemmy_item::<GroupInboxActivities>(
      "assets/lemmy/activities/create_or_update/create_note.json",
    )
    .unwrap();
  }

  #[test]
  fn test_person_inbox() {
    test_parse_lemmy_item::<PersonInboxActivities>("assets/lemmy/activities/following/accept.json")
      .unwrap();
    test_parse_lemmy_item::<PersonInboxActivitiesWithAnnouncable>(
      "assets/lemmy/activities/create_or_update/create_note.json",
    )
    .unwrap();
    test_parse_lemmy_item::<PersonInboxActivitiesWithAnnouncable>(
      "assets/lemmy/activities/create_or_update/create_private_message.json",
    )
    .unwrap();
    test_json::<PersonInboxActivitiesWithAnnouncable>("assets/mastodon/activities/follow.json")
      .unwrap();
  }

  #[test]
  fn test_site_inbox() {
    test_parse_lemmy_item::<SiteInboxActivities>(
      "assets/lemmy/activities/deletion/delete_user.json",
    )
    .unwrap();
  }
}
