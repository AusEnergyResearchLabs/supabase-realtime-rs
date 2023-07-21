-- Taken from https://github.com/supabase/realtime/blob/main/dev/postgres/00-setup.sql
create role anon          nologin noinherit;
create role authenticated nologin noinherit;
create role service_role  nologin noinherit bypassrls;

grant usage on schema public to anon, authenticated, service_role;

alter default privileges in schema public grant all on tables    to anon, authenticated, service_role;
alter default privileges in schema public grant all on functions to anon, authenticated, service_role;
alter default privileges in schema public grant all on sequences to anon, authenticated, service_role;

create schema if not exists _realtime;
create schema if not exists realtime;

create publication supabase_realtime with (publish = 'insert, update, delete');
