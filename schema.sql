CREATE TYPE day_type AS ENUM (
    'monday',
    'tuesday',
    'wednesday',
    'thursday',
    'friday',
    'saturday',
    'sunday'
);

CREATE TYPE week_type AS ENUM ('odd', 'even');

CREATE TABLE majors (
    id text PRIMARY KEY,
    title text,
    enrollment_year smallint
);

CREATE TABLE timetable (
    id bigint GENERATED ALWAYS AS identity (minvalue 1000) PRIMARY KEY,
    major text NOT NULL,
    week week_type NOT NULL,
    day_of_week day_type NOT NULL,
    starts_at time NOT NULL,
    ends_at time NOT NULL,
    subject_name text NOT NULL,
    subject_type text NOT NULL,
    auditorium text NOT NULL,
    professor text,

    UNIQUE(week, day_of_week, starts_at)
);

CREATE TABLE users (
    id bigint PRIMARY KEY,
    major_id text,
    CONSTRAINT fk_major
        FOREIGN KEY (major_id)
            REFERENCES majors(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);
