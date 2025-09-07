/// This module contains the admin functions that can be called by the admin
/// The admin is the principal that can call the admin functions
/// The admin is set when the house is initialized
/// The admin can be changed by calling the setAdmin function
///
/// the admin can manually initiate the collection of borrow fees by calling the collectBorrowFees function
/// this also in turn collects the fundingfees
pub mod admin_functions;

pub mod roles;
