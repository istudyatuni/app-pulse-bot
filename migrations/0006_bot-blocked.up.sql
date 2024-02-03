-- this columns stores is user blocked bot
alter table user add column bot_blocked int not null default false;
