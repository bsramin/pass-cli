/*
 *  Copyright (c) 2026 Proton AG
 *  This file is part of Proton AG and Proton Pass.
 *
 *  Proton Pass is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  Proton Pass is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with Proton Pass.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::common::CodeResponse;
use crate::{PassClient, PassClientContext};
use anyhow::{Context, Result};
use muon::PUT;
use pass_domain::{ItemId, ShareId};

#[derive(Debug, serde::Serialize)]
struct UpdateLastUseTimeRequest {
    #[serde(rename = "LastUseTime")]
    last_use_time: i64,
}

impl<C: PassClientContext> PassClient<C> {
    pub async fn update_item_last_use_time(
        &self,
        share_id: &ShareId,
        item_id: &ItemId,
    ) -> Result<()> {
        let request = UpdateLastUseTimeRequest {
            last_use_time: jiff::Timestamp::now().as_second(),
        };

        let req = PUT!("/pass/v1/share/{share_id}/item/{item_id}/lastuse")
            .body_json(request)
            .context("Error serializing last use time request")?;

        let res = self
            .send(req)
            .await
            .context("Failed to send last use time request")?;

        let response: CodeResponse = assert_response!(res);
        response.success_guard()?;

        self.clear_items_cache(share_id).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_tools::*;

    #[muon_test::test]
    async fn test_update_item_last_use_time(server: muon_test::Server) {
        let (raw_client, api) = server.client::<()>();
        const SHARE_ID: &str = "MyShareID";
        const ITEM_ID: &str = "MyItemID";

        let client = make_test_pass_client_with_setup(raw_client, &api, PlanType::Free).await;

        let handled = api.handler_with_method(
            Method::PUT,
            format!("/pass/v1/share/{SHARE_ID}/item/{ITEM_ID}/lastuse"),
            move |_| success_code(),
        );

        client
            .update_item_last_use_time(&share_id!(SHARE_ID), &item_id!(ITEM_ID))
            .await
            .expect("Should be able to update the last use time");

        assert_hit!(handled);
    }
}
