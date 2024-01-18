alter table user drop column last_version_notified;
-- this columns store last app version, about which user was notified
alter table user add column last_version_notified integer default 0;
