-- Deploy monk:db to pg
BEGIN;

SET client_min_messages TO WARNING;

CREATE SCHEMA IF NOT EXISTS public;

COMMIT;
