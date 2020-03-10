use crate::connection::{Connection, ConnectionId};

use std::collections::HashMap;
use std::collections::hash_map::{Entry, Iter, IterMut};

pub struct ConnectionPool {
    inner: HashMap<ConnectionId, Connection>,
}

impl ConnectionPool {

    /// Creates a new `ConnectionPool`.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Returns the number of connections stored inside of the pool.
    pub fn size(&self) -> usize {
        self.inner.len()
    }

    /// Inserts a `Connection` if it hasn't been added before. Returns `true` if the insertion
    /// succeeds, otherwise `false`.
    ///
    /// TODO: instead of ignoring the insertion attempt, if a connection with the same ID already
    /// exists, should we replace?
    pub fn insert(&mut self, conn: Connection) -> bool {
        match self.inner.entry(conn.id.clone()) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(conn);
                true
            }
        }
    }

    /// Removes a `Connection` with the given `ConnectionId`. Returns `true` if the removal
    /// succeeds, otherwise `false`.
    pub fn remove(&mut self, conn_id: &ConnectionId) -> bool {
        self.inner.remove(conn_id).is_some()
    }

    /// Returns a shared reference to a `Connection` with the given `ConnectionId`.
    pub fn get(&self, conn_id: &ConnectionId) -> Option<&Connection> {
        self.inner.get(conn_id)
    }

    /// Returns a mutable reference to a `Connection` with the given `ConnectionId`.
    pub fn get_mut(&mut self, conn_id: &ConnectionId) -> Option<&mut Connection> {
        self.inner.get_mut(conn_id)
    }

    /// Returns an iterator to visit all connections immutably.
    pub fn iter(&self) -> Iter<ConnectionId, Connection> {
        self.inner.iter()
    }

    /// Returns an iterator to visit all connections mutably.
    pub fn iter_mut(&mut self) -> IterMut<ConnectionId, Connection> {
        self.inner.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::url::Url;
    use crate::connection::ConnectionState;
    use async_std::task::block_on;

    #[test]
    fn insert_and_remove_from_pool() {
        let mut pool = ConnectionPool::new();
        assert_eq!(0, pool.size(), "Incorrect pool size");

        let url = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let conn = Connection::from_url(url);
        let conn_id = conn.id.clone();

        assert!(pool.insert(conn), "Insertion failed");
        assert_eq!(1, pool.size(), "Incorrect pool size");

        assert!(pool.remove(&conn_id));
        assert_eq!(0, pool.size(), "Incorrect pool size");
    }

    #[test]
    fn iterate_pool() {
        let mut pool = ConnectionPool::new();

        let url1 = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let conn1 = Connection::from_url(url1);

        let url2 = block_on(Url::from_str_with_port("udp://localhost:17000")).unwrap();
        let conn2 = Connection::from_url(url2);

        assert!(pool.insert(conn1), "Insertion failed");
        assert!(pool.insert(conn2), "Insertion failed");
        assert_eq!(2, pool.size(), "Incorrect pool size");

        let mut count = 0;
        for (_id, _conn) in pool.iter() {
            count += 1;
        }
        assert_eq!(2, count, "Immutable iteration failed");

        let mut count = 0;
        for (_id, _conn) in pool.iter_mut() {
            count += 1;
        }
        assert_eq!(2, count, "Mutable iteration failed");
    }

    #[test]
    fn get_connection_from_pool() {
        let mut pool = ConnectionPool::new();

        let url1 = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let conn1 = Connection::from_url(url1);
        let conn1_id = conn1.id.clone();

        pool.insert(conn1);

        assert!(pool.get(&conn1_id).is_some(), "Getting connection from pool failed");
    }

    #[test]
    fn get_mutable_connection_from_pool() {
        let mut pool = ConnectionPool::new();

        let url1 = block_on(Url::from_str_with_port("udp://localhost:16000")).unwrap();
        let conn1 = Connection::from_url(url1);
        let conn1_id = conn1.id.clone();

        pool.insert(conn1);

        assert!(pool.get_mut(&conn1_id).is_some(), "Getting connection from pool failed");

        if let Some(conn1) = pool.get_mut(&conn1_id) {
            assert_eq!(ConnectionState::Awaited, conn1.state);
            conn1.state = ConnectionState::Stalled;
            assert_eq!(ConnectionState::Stalled, conn1.state);
        } else {
            assert!(false, "Getting mutable connection from pool failed");
        }
    }
}