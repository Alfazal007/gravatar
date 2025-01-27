create table profile (
	id bigint primary key,
	user_id bigint references users(id) not null
);
