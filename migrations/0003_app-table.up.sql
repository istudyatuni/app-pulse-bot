create table app (
	app_id text,
	source_id int,
	name text,
	last_updated_at int default 0, -- unix time

	primary key (app_id, source_id)
);
