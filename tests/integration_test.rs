use psu::{create_table, insert, remove, search_by_id, Password};
use rusqlite::Connection;

#[test]
fn is_insert() {
    let conn = Connection::open("tests/tests.db").unwrap();

    create_table(&conn).unwrap();

    let p = Password {
        id: 1,
        service: "gog".to_string(),
        login: "man".to_string(),
        password: "1322".to_string(),
    };

    insert(&conn, &p.service, &p.login, &p.password).unwrap();

    let result = search_by_id(&conn, 1).unwrap();

    conn.execute("DELETE FROM password", []).unwrap();

    assert_eq!(result, p)
}

#[test]
fn is_modify() {
    let conn = Connection::open("tests/tests.db").unwrap();

    create_table(&conn).unwrap();

    let r = Password {
        id: 1,
        service: "gog".to_string(),
        login: "man".to_string(),
        password: "1322".to_string(),
    };

    insert(&conn, &r.service, &r.login, &r.password).unwrap();

    let p = Password {
        id: 1,
        password: "1".to_string(),
        ..r.clone()
    };

    p.modify(&conn);

    let result = search_by_id(&conn, 1).unwrap();

    conn.execute("DELETE FROM password", []).unwrap();

    assert_eq!(result, p)
}

#[test]
fn is_remove() {
    let conn = Connection::open("tests/tests.db").unwrap();

    create_table(&conn).unwrap();

    let p = Password {
        id: 1,
        service: "gog".to_string(),
        login: "man".to_string(),
        password: "1322".to_string(),
    };

    insert(&conn, &p.service, &p.login, &p.password).unwrap();

    remove(&conn, Some(1), false);

    let result = search_by_id(&conn, 1);

    conn.execute("DELETE FROM password", []).unwrap();

    assert_eq!(result, None)
}

#[test]
fn is_remove_all() {
    let conn = Connection::open("tests/tests.db").unwrap();

    create_table(&conn).unwrap();

    let p = Password {
        id: 1,
        service: "gog".to_string(),
        login: "man".to_string(),
        password: "1322".to_string(),
    };

    insert(&conn, &p.service, &p.login, &p.password).unwrap();

    remove(&conn, None, true);

    let result = search_by_id(&conn, 1);

    assert_eq!(result, None)
}
