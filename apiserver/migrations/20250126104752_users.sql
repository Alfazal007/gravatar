create table users (
	id serial primary key,
	email varchar(30) not null unique,
	password varchar(255) not null,
	email_hash varchar(255) not null,
	active_photo_id int default -1
);
