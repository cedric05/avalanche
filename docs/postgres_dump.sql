
CREATE TABLE project (
    id integer NOT NULL,
    index character varying NOT NULL,
    needs_auth boolean NOT NULL
);

CREATE SEQUENCE project_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER TABLE ONLY project ALTER COLUMN id SET DEFAULT nextval('project_id_seq'::regclass);


ALTER TABLE ONLY project
    ADD CONSTRAINT project_pkey PRIMARY KEY (id);



CREATE TABLE subproject (
    id integer NOT NULL,
    project_id integer NOT NULL,
    method jsonb NOT NULL,
    query_params jsonb NOT NULL,
    headers jsonb NOT NULL,
    auth jsonb NOT NULL,
    params jsonb NOT NULL,
    index text NOT NULL,
    url character varying NOT NULL
);

CREATE SEQUENCE subproject_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER TABLE ONLY subproject ALTER COLUMN id SET DEFAULT nextval('subproject_id_seq'::regclass);

ALTER TABLE ONLY subproject
    ADD CONSTRAINT subproject_pkey PRIMARY KEY (id);



ALTER TABLE project OWNER TO postgres;
ALTER TABLE project_id_seq OWNER TO postgres;
ALTER SEQUENCE project_id_seq OWNED BY project.id;
ALTER TABLE subproject OWNER TO postgres;

ALTER TABLE subproject_id_seq OWNER TO postgres;
ALTER SEQUENCE subproject_id_seq OWNED BY subproject.id;
