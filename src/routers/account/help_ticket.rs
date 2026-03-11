use diesel::prelude::*;
use salvo::prelude::*;

use crate::models::help_ticket::*;
use crate::models::*;
use crate::schema::*;
use crate::{context, db, AppResult};

#[handler]
pub async fn show(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let ticket_id = get_id_param!(req, res);
    let cuser = current_user!(depot, res);

    let mut conn = db::connect()?;

    let help_ticket_option = help_tickets::table
        .filter(
            help_tickets::owner_id
                .eq(cuser.id)
                .and(help_tickets::id.eq(ticket_id)),
        )
        .first::<HelpTicket>(&mut conn)
        .ok();

    if let Some(help_ticket) = help_ticket_option {
        res.render(Json(help_ticket));
    } else {
        return context::render_not_found_json(res);
    }
    Ok(())
}

#[handler]
pub async fn list(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    let cuser = current_user!(depot, res);
    let query = help_tickets::table.filter(help_tickets::owner_id.eq(cuser.id));
    let mut conn = db::connect()?;
    list_records!(
        req,
        res,
        HelpTicket,
        query,
        "updated_at desc",
        HELP_TICKET_FILTER_FIELDS.clone(),
        HELP_TICKET_JOINED_OPTIONS.clone(),
        ID_NAME_SEARCH_TMPL,
        &mut conn
    );
    Ok(())
}
