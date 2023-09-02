///////////////////////////////////////////////////////////////
/* db version: v1.0.0-beta.10 */
///////////////////////////////////////////////////////////////
/* auth */
DEFINE TABLE auth SCHEMAFULL;

DEFINE FIELD username ON TABLE auth TYPE string ASSERT $value = /[a-z][a-z0-9_]{1,20}/;
DEFINE FIELD password ON TABLE auth TYPE string;
DEFINE FIELD email ON TABLE auth TYPE string VALUE string::lowercase($value) ASSERT is::email($value);
DEFINE FIELD is_email_verified ON TABLE auth TYPE bool DEFAULT false;
DEFINE FIELD create_time ON TABLE auth TYPE datetime DEFAULT time::now();
DEFINE FIELD update_time ON TABLE auth TYPE option<datetime>;
DEFINE FIELD last_login_time ON TABLE auth TYPE datetime;

DEFINE INDEX authUsernameIndex ON TABLE auth COLUMNS username UNIQUE;
DEFINE INDEX authEmailIndex ON TABLE auth COLUMNS email UNIQUE;

///////////////////////////////////////////////////////////////
/* user */
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON TABLE user TYPE string ASSERT string::len($value) <= 30;
DEFINE FIELD description ON TABLE user TYPE string ASSERT string::len($value) <= 255;
DEFINE FIELD avatar ON TABLE user TYPE option<record<attachment>>;
DEFINE FIELD region_code ON TABLE user TYPE string;
DEFINE FIELD language_code ON TABLE user TYPE string;
DEFINE FIELD time_zone ON TABLE user TYPE string;
DEFINE FIELD create_time ON TABLE user TYPE datetime DEFAULT time::now();
DEFINE FIELD update_time ON TABLE uesr TYPE option<datetime>;


///////////////////////////////////////////////////////////////
/* server */
DEFINE TABLE server SCHEMAFULL;

DEFINE FIELD display_name ON TABLE server TYPE string ASSERT string::len($value) <= 50;
DEFINE FIELD description ON TABLE server TYPE string ASSERT string::len($value) <= 255;
DEFINE FIELD owner ON TABLE server TYPE record<user>;
DEFINE FIELD author ON TABLE server TYPE record<user>;
DEFINE FIELD icon ON TABLE server TYPE option<record<attachment>>;
DEFINE FIELD create_time ON TABLE server TYPE datetime DEFAULT time::now();
DEFINE FIELD update_time ON TABLE server TYPE option<datetime>;

///////////////////////////////////////////////////////////////
/* category */
DEFINE TABLE category SCHEMAFULL;

DEFINE FIELD display_name ON TABLE category TYPE string ASSERT string::len($value) <= 50;
DEFINE FIELD description ON TABLE category TYPE string ASSERT string::len($value) <= 255;
DEFINE FIELD server_id ON TABLE category TYPE record<server>;
DEFINE FIELD icon ON TABLE category TYPE option<record<attachment>>;
DEFINE FIELD order ON TABLE category TYPE int ASSERT $value >= 0;
DEFINE FIELD create_time ON TABLE category TYPE datetime DEFAULT time::now();
DEFINE FIELD update_time ON TABLE category TYPE option<datetime>;

///////////////////////////////////////////////////////////////
/* channel */
DEFINE TABLE channel SCHEMAFULL;

DEFINE FIELD display_name ON TABLE channel TYPE string ASSERT string::len($value) <= 50;
DEFINE FIELD description ON TABLE channel TYPE string ASSERT string::len($value) <= 255;
DEFINE FIELD channel_type ON TABLE channel TYPE string ASSERT string::len($value) <= 255;
DEFINE FIELD icon ON TABLE channel TYPE option<record<attachment>>;
DEFINE FIELD channel_type ON TABLE channel TYPE bool VALUE $value INSIDE ["SAVED", "DIRECT", "SERVER"];
DEFINE FIELD order ON TABLE channel TYPE int ASSERT $value >= 0;
DEFINE FIELD create_time ON TABLE channel TYPE datetime DEFAULT time::now();
DEFINE FIELD update_time ON TABLE channel TYPE option<datetime>;

///////////////////////////////////////////////////////////////
/* attachment */
DEFINE TABLE attachment SCHEMAFULL;

DEFINE FIELD url ON TABLE attachment TYPE string ASSERT ASSERT is::url($value);
DEFINE FIELD filename ON TABLE attachment TYPE string;
DEFINE FIELD mime_type ON TABLE attachment TYPE string;
DEFINE FIELD file_size ON TABLE attachment TYPE number ASSERT $value > 0 AND is::int($value); // unit: bytes
DEFINE FIELD metadata ON TABLE attachment FLEXIBLE TYPE option<object>;
DEFINE FIELD create_time ON TABLE attachment TYPE datetime DEFAULT time::now();


///////////////////////////////////////////////////////////////
/* message */
DEFINE TABLE message SCHEMAFULL;

DEFINE FIELD author ON TABLE message TYPE record<user>;
DEFINE FIELD owner ON TABLE message TYPE record<user>;
DEFINE FIELD content ON TABLE message TYPE record<user>;
DEFINE FIELD attachments ON TABLE message TYPE array<record<attachment>>;
DEFINE FIELD create_time ON TABLE message TYPE datetime DEFAULT time::now();
DEFINE FIELD update_time ON TABLE message TYPE option<datetime>;

///////////////////////////////////////////////////////////////
/* reaction */
// use 'RELATE' statement
// in: user
// out: message