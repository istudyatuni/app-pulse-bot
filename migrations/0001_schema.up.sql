create table user (
	user_id int primary key,
	lang text not null,
	last_notified_at int default 0 -- unix time
);

create table user_update (
	user_id int not null,
	source_id int not null,
	app_id text not null,
	should_notify int not null, -- bool

	primary key (user_id, source_id, app_id)
);

create table source (
	source_id int primary key,
	name text,
	last_updated_at int default 0 -- unix time
);

create table user_subscribe (
	user_id int not null,
	source_id int not null,
	subscribed int not null, -- bool

	primary key (user_id, source_id)
);
