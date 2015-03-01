CREATE EXTENSION IF NOT EXISTS citext;
CREATE OR REPLACE LANGUAGE plpgsql;

DROP TABLE IF EXISTS Library;
DROP TABLE IF EXISTS Entry;
DROP TABLE IF EXISTS Game;
DROP TABLE IF EXISTS Login;
DROP TYPE IF EXISTS Status;

CREATE TYPE Status AS ENUM (
	'Frozen',
	'CurrentlyPlaying',
	'Dropped',
	'PlanToPlay'
);

CREATE TABLE Login (
	id SERIAL PRIMARY KEY,
	username VARCHAR(20) NOT NULL UNIQUE,
	password VARCHAR(128) NOT NULL,
	email CITEXT NOT NULL UNIQUE
);

CREATE TABLE Game (
	id SERIAL PRIMARY KEY,
	name TEXT NOT NULL,
	description TEXT NOT NULL
);

CREATE TABLE Entry (
	id SERIAL PRIMARY KEY,
	game_id INT NOT NULL,
	time_played REAL NOT NULL,
	last_update TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	status STATUS NOT NULL,
	FOREIGN KEY (game_id) REFERENCES Game(id)
);

CREATE TABLE Library (
	id SERIAL PRIMARY KEY,
	login_id INT NOT NULL,
	entry_id INT NOT NULL UNIQUE,
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


INSERT INTO Login (username, password, email) VALUES ('user', 'hunter2', 'user@example.com');
INSERT INTO Game (name, description) VALUES
	('Diablo III', 'Two decades have passed since the demonic denizens, Diablo, Mephisto, and Baal, wandered the world of Sanctuary in a vicious rampage to shackle humanity into unholy slavery. Yet for those who battled the Prime Evils directly, the memory fades slowly and the wounds of the soul still burn.

	When Deckard Cain returns to the ruins of Tristram''s Cathedral seeking clues to new stirrings of evil, a comet from the heavens strikes the very ground where Diablo once entered the world. The comet carries a dark omen in its fiery being and it calls the heroes of Sanctuary to defend the mortal world against the rising powers of the Burning Hells and even the failing luminaries of the High Heavens itself.'),
	('Darksiders', 'Deceived by the forces of evil into prematurely bringing about the end of the world, War – the first Horseman of the Apocalypse stands accused of breaking the sacred law by inciting a war between Heaven and Hell. In the ensuing slaughter, demonic forces defeated the Heavenly Hosts and laid claim to the Earth. Brought before the sacred Charred Council, War is indicted for his crimes and stripped of his powers. Dishonored and facing his own demise, War is given the opportunity to find redemption by returning to Earth to search for the truth—and to punish those responsible. Hunted by vengeful Angels, War must forge uneasy alliances with the very demons he hunts as he journeys across the ravaged remains of Earth on his quest for vindication. The answers he seeks will lead him to take on the forces of Hell, and reveals a conspiracy in which War is just a pawn in the eternal battle between far greater powers.'),
	('Bioshock Infinite', 'BioShock Infinite is a first-person shooter made by Irrational Games, the studio behind the original BioShock (which sold over 4 million units worldwide). Set in 1912, BioShock Infinite introduces an entirely new narrative and gameplay experience that lifts players out of the familiar confines of Rapture and rockets them to Columbia, an immense city in the sky. Former Pinkerton agent Booker DeWitt has been sent to rescue Elizabeth, a young woman imprisoned in Columbia since childhood. Booker develops a relationship with Elizabeth, augmenting his abilities with hers so the pair may escape from a city that is literally falling from the sky. DeWitt must learn to fight foes in high-speed Sky-Line battles, engage in combat both indoors and amongst the clouds, and harness the power of dozens of new weapons and abilities.'),
	('Skyrim', 'You should have acted. They''re already here. The Elder Scrolls told of their return. Their defeat was merely delay until the time after Oblivion opened, when the sons of Skyrim would spill their own blood. But... there is one they fear. In their tongue he is Dovahkiin; Dragon Born!'),
	('Mass Effect 3', 'Plunges you into an all-out galactic war to take Earth back from a nearly unstoppable foe - and how you fight that war is entirely up to you.'),
	('The Witcher 3', 'The war with Nilfgaard obliterated the old order. The North is engulfed in chaos, and marching armies leave a plague of monsters in their wake. Geralt of Rivia once more treads the Witcher’s Path.'),
	('Darksiders II', 'Awakened by the End of Days, Death, the most feared of the legendary Four Horsemen, embarks upon a quest to restore mankind, and redeem his brother’s name. Along the way, the Horseman discovers that there are far worse things than an earthly Apocalypse, and that an ancient grudge may threaten all of Creation.'),
	('The Witcher 2', 'The second instalment in the RPG saga about the Witcher, Geralt of Rivia, features a thoroughly engrossing, mature storyline defining new standards for thought-provoking, non-linear game narration. In addition to an epic story, the game features an original, brutal combat system that uniquely combines tactical elements with dynamic action.

		A new, modern game engine, responsible both for beautiful visuals and sophisticated game mechanics puts players in the most lively and believable world ever created in an RPG game. A captivating story, dynamic combat system, beautiful graphics, and everything else that made the original Witcher such a great game are now executed in a much more advanced and sophisticated way.'),
	('The Witcher', 'Welcome to a world that knows no mercy - none is received, and none is given. Only physical and mental agility can keep you alive, though they are by no means a guarantor of life. You play the role of Geralt, an already legendary monster-slayer - but this, my friend, is not a gift, and is certainly not given lightly. “The Witcher” is an immense computer game. Within its realm, you will have to assume the burden of choice. And this burden of choice, as light as it may appear, is the very thing that will both permit you to wield influence over the fate of the world, as well as get you slain prematurely.')
;
