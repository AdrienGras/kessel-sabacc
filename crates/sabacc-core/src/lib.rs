//! Core game logic for Kessel Sabacc.
//!
//! This crate contains pure game logic with zero I/O and zero UI dependencies.
//! All randomness is passed in as a parameter for deterministic testing.

pub mod bot;
pub mod card;
pub mod deck;
pub mod error;
pub mod game;
pub mod hand;
pub mod player;
pub mod round;
pub mod scoring;
pub mod shift_token;
pub mod turn;

/// Unique identifier for a player in the game.
pub type PlayerId = u8;
