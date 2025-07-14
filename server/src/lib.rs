use spacetimedb::{table, reducer, Table, ReducerContext, Identity, Timestamp};

#[table(name = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,
    name: Option<String>,
    online: bool,
}

#[table(name = message, public)]
pub struct Message {
    sender: Identity,
    sent: Timestamp,
    text: String,
}

#[reducer]
/// Clients invoke this reducer to set their user names.
pub fn set_name(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let name = validate_name(name)?;

    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        ctx.db.user().identity().update(User {name: Some(name), ..user});
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}

#[reducer]
/// Clients invoke this reducer to send messages.
pub fn send_message(ctx: &ReducerContext, text: String) -> Result<(), String> {
    let text = validate_message(text)?;
    log::info!("{}: {}", ctx.sender, text);
    ctx.db.message().insert(Message {
        sender: ctx.sender,
        text,
        sent: ctx.timestamp,
    });
    Ok(())
}

#[reducer(client_connected)]
/// Called when a client connects to the SpaceTimeDB database server.
pub fn client_connected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        // If this is a returning user, i.e we already have a `User` with this `Identity`,
        // set `online` to true, but leave `name` and `identity` unchanged.
        ctx.db.user().identity().update(User { online: true, ..user });
    } else {
        // If this is a new user, create a new `User` with the `Identity` and set `online` to true, but hasn't set a name yet.
        ctx.db.user().insert(User {
            
            identity: ctx.sender,
            name: None,
            online: true,
        });
    }
}

#[reducer(client_disconnected)]
/// Called when a client disconnects from SpacetimeDB database server
pub fn identity_disconnected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        ctx.db.user().identity().update(User { online: false, ..user });
    } else {
        // This branch should be unreachable,
        // as it doesn't make sense for a client to disconnect without connecting first.
        log::warn!("Disconnect event for unknown user with identity {:?}", ctx.sender);
    }
}



/// Takes a name and checks if it's acceptable as a user's name
fn validate_name(name: String) -> Result<String, String> {
    if name.is_empty() {
        Err("Name cannot be empty".to_string())
    } else if name.len() > 100 {
        Err("Name is too long".to_string())
    } else if name.contains('\n') {
        Err("Name cannot contain newlines".to_string())
    } else {
        Ok(name)
    }
}

/// Takes a message and checks if it's acceptable as a user's message
fn validate_message(text: String) -> Result<String, String> {
    if text.is_empty() {
        Err("Message cannot be empty".to_string())
    } else if text.len() > 1000 {
        Err("Message is too long".to_string())
    } else {
        Ok(text)
    }
}
