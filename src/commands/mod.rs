use anyhow::Result;
use bitcoin::address::NetworkUnchecked;
use bitcoin::Address;
use bitcoin::Amount;
use bitcoin::OutPoint;
use clap::{Parser, Subcommand};
use ord::{Chain, FeeRate, Inscription, InscriptionId};

pub mod inscribe_generator;
pub mod mint;
