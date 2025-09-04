pub mod user;
pub mod team;
pub mod team_invite;

/*
 Users can exist alone but have no access unless in a team. Creating an account is "free" but you have no access otherwise
 Creating a team costs seats + resource usage. Always needs 1 owner
 Users can be invited to teams without being owners thus giving them access
 so the flow would be:
 Noah signs up and creates a free account. No access.
 Loudbook signs up and creates a team pays $6 for 2 seats and invites Noah.
 Noah's api key has power now in Loudbook's team
 */
