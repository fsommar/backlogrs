CREATE EXTENSION IF NOT EXISTS citext;
CREATE OR REPLACE LANGUAGE plpgsql;

DROP TYPE IF EXISTS Status;
DROP TABLE IF EXISTS Login;
DROP TABLE IF EXISTS Game;
DROP TABLE IF EXISTS Entry;
DROP TABLE IF EXISTS Library;

CREATE TYPE Status AS ENUM (
	'Frozen',
	'CurrentlyPlaying',
	'Dropped',
	'PlanToPlay'
);

CREATE TABLE Login (
	id			SERIAL PRIMARY KEY,
	username	VARCHAR(20) NOT NULL UNIQUE,
	password	VARCHAR(20),
	email		CITEXT UNIQUE
);

CREATE TABLE Game (
	id			SERIAL PRIMARY KEY,
	name		TEXT NOT NULL,
	description	TEXT NOT NULL
);

CREATE TABLE Entry (
	id			SERIAL PRIMARY KEY,
	game_id		INT NOT NULL,
	time_played	REAL NOT NULL,
	last_update	TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	status		STATUS NOT NULL,
	FOREIGN KEY (game_id) REFERENCES Game(id)
);

CREATE TABLE Library (
	id			SERIAL PRIMARY KEY,
	login_id	INT NOT NULL,
	entry_id	INT NOT NULL,
	FOREIGN KEY (login_id) REFERENCES Login(id),
	FOREIGN KEY (entry_id) REFERENCES Entry(id)
);

CREATE OR REPLACE FUNCTION update_last_update()
RETURNS TRIGGER AS $$
BEGIN
	NEW.last_update = now();
	RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER entry_last_update BEFORE UPDATE
ON Entry FOR EACH ROW EXECUTE PROCEDURE
update_last_update();
