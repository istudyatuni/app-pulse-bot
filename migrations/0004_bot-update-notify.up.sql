-- this columns store last app version, about which user was notified
alter table user add column last_version_notified text default '';
