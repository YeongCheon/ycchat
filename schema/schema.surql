///////////////////////////////////////////////////////////////
/* db version: v1.0.0-beta.10 */
///////////////////////////////////////////////////////////////
/* auth */
DEFINE TABLE auth SCHEMAFULL;

DEFINE FIELD username ON auth TYPE string ASSERT $value = /[a-z][a-z0-9_]{1,20}/;
DEFINE FIELD password ON auth TYPE string;
DEFINE FIELD email ON auth TYPE string VALUE string::lowercase($value) ASSERT is::email($value);
DEFINE FIELD is_email_verified ON auth TYPE bool DEFAULT false;
DEFINE FIELD create_time ON auth TYPE datetime DEFAULT time::now();
DEFINE FIELD update_time ON auth TYPE option<datetime>;
DEFINE FIELD last_login_time ON auth TYPE datetime;

DEFINE INDEX authUsernameIndex ON auth COLUMNS username UNIQUE;
DEFINE INDEX authEmailIndex ON auth COLUMNS email;

///////////////////////////////////////////////////////////////
/* user */
DEFINE TABLE user SCHEMAFULL;

DEFINE FIELD display_name ON user TYPE string ASSERT string::len($value) <= 30;
DEFINE FIELD description ON user TYPE string ASSERT string::len($value) <= 255;
DEFINE FIELD avatar ON user TYPE option<record<attachment>>;
DEFINE FIELD region_code ON user TYPE string;
DEFINE FIELD language_code ON user TYPE string;
DEFINE FIELD time_zone ON user TYPE string;
DEFINE FIELD create_time ON user TYPE datetime DEFAULT time::now();
DEFINE FIELD update_time ON user TYPE option<datetime>;

///////////////////////////////////////////////////////////////
/* server */
DEFINE TABLE server SCHEMAFULL;

DEFINE FIELD display_name ON server TYPE string ASSERT string::len($value) <= 50;
DEFINE FIELD description ON server TYPE string ASSERT string::len($value) <= 255;
DEFINE FIELD owner ON server TYPE record<user>;
DEFINE FIELD author ON server TYPE record<user>;
DEFINE FIELD icon ON server TYPE option<record<attachment>>;
DEFINE FIELD create_time ON server TYPE datetime DEFAULT time::now();
DEFINE FIELD update_time ON server TYPE option<datetime>;
DEFINE FIELD managers ON server TYPE array<record<user>>;

///////////////////////////////////////////////////////////////
/* category */
DEFINE TABLE category SCHEMAFULL;

DEFINE FIELD display_name ON category TYPE string ASSERT string::len($value) <= 50;
DEFINE FIELD description ON category TYPE string ASSERT string::len($value) <= 255;
DEFINE FIELD server ON category TYPE record<server>;
DEFINE FIELD icon ON category TYPE option<record<attachment>>;
DEFINE FIELD order ON category TYPE int ASSERT $value >= 0;
DEFINE FIELD create_time ON category TYPE datetime DEFAULT time::now();
DEFINE FIELD update_time ON category TYPE option<datetime>;

///////////////////////////////////////////////////////////////
/* channel */
DEFINE TABLE channel SCHEMAFULL;

DEFINE FIELD channel_type ON channel TYPE string
  ASSERT $value INSIDE ["SAVED", "DIRECT", "GROUP", "SERVER"];
DEFINE FIELD display_name ON channel TYPE string ASSERT string::len($value) <= 50;
DEFINE FIELD description ON channel TYPE string ASSERT string::len($value) <= 255;
DEFINE FIELD icon ON channel TYPE option<record<attachment>>;
DEFINE FIELD order ON channel TYPE int ASSERT $value >= 0;
DEFINE FIELD create_time ON channel TYPE datetime DEFAULT time::now();
DEFINE FIELD update_time ON channel TYPE option<datetime>;

DEFINE FIELD server ON channel TYPE option<record<server>>;
DEFINE FIELD category ON channel TYPE option<record<category>>;
DEFINE FIELD members ON channel TYPE array<record<user>>; // only use when channel_type field is not 'SERVER'.

///////////////////////////////////////////////////////////////
/* attachment */
DEFINE TABLE attachment SCHEMAFULL;

DEFINE FIELD url ON attachment TYPE string ASSERT is::url($value);
DEFINE FIELD filename ON attachment TYPE string;
DEFINE FIELD mime_type ON attachment TYPE string;
DEFINE FIELD file_size ON attachment TYPE number ASSERT $value > 0 AND type::int($value); // unit: bytes
DEFINE FIELD metadata ON attachment FLEXIBLE TYPE option<object>;
DEFINE FIELD create_time ON attachment TYPE datetime DEFAULT time::now();


///////////////////////////////////////////////////////////////
/* message */
DEFINE TABLE message SCHEMAFULL;

DEFINE FIELD author ON message TYPE record<user>;
DEFINE FIELD content ON message TYPE record<user>;
DEFINE FIELD attachments ON message TYPE array<record<attachment>>;
DEFINE FIELD create_time ON message TYPE datetime DEFAULT time::now();
DEFINE FIELD update_time ON message TYPE option<datetime>;

///////////////////////////////////////////////////////////////
/* member */
DEFINE TABLE member SCHEMAFULL;

DEFINE FIELD display_name ON member TYPE string ASSERT string::len($value) <= 50;
DEFINE FIELD description ON member TYPE string ASSERT string::len($value) <= 255;
DEFINE FIELD avatar ON member TYPE option<record<attachment>>;
DEFINE FIELD update_time ON member TYPE option<datetime>;
DEFINE FIELD create_time ON member TYPE datetime;

DEFINE INDEX unique_member ON member COLUMNS in, out UNIQUE;

///////////////////////////////////////////////////////////////
/* reaction */
// RELATE user:USER_ID->reaction->message:MESSAGE_ID
DEFINE TABLE reaction SCHEMAFULL;

DEFINE FIELD create_time ON reaction TYPE datetime;

DEFINE INDEX unique_reaction ON reaction COLUMNS in, out UNIQUE;