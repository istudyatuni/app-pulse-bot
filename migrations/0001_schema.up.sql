create table user (
	user_id int primary key,
	lang text not null
);

create table user_update (
	user_id int not null,
	app_id text not null,
	should_notify int not null, -- bool

	primary key (user_id, app_id)
);
