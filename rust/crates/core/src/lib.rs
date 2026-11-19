//! Domain rules with no I/O. Everything here is a pure function of its inputs, so it is tested
//! without a database, a clock or the network.

pub mod calendar;

pub mod grade;

pub mod stars;

pub mod streak;

pub mod subject;
