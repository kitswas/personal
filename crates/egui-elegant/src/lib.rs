//! # egui-elegant
//!
//! A beautiful, minimal, and elegant UI component library for `egui`.
//!
//! This crate provides a suite of ready-to-use widgets and a theming engine
//! that make it easy to build stunning immediate-mode GUIs in Rust.
//!
//! ## Features
//! - **Theming**: See [`theme::ElegantTheme`] for setting up dark/light modes and colors.
//! - **Components**: Buttons, Badges, Cards, Accordions, Alerts, Dropdowns, Inputs,
//!   Progress Bars, Skeletons, and Tabs.
//! - **Flex Layouts**: Enable the `flex` feature to seamlessly integrate with
//!   `egui_flex`.
//!
//!
//! ## Example Usage
//! 
//! ```rust,no_run
//! use eframe::egui;
//! use egui_elegant::{ElegantTheme, ThemeMode, MonaspaceFont, ElegantButton, Variant};
//! 
//! fn main() -> Result<(), eframe::Error> {
//!     eframe::run_simple_native("App", Default::default(), |ctx, _frame| {
//!         // 1. Apply the elegant theme
//!         let theme = ElegantTheme::build(ThemeMode::System, MonaspaceFont::Neon);
//!         theme.apply(ctx);
//!         
//!         // 2. Build your UI
//!         egui::CentralPanel::default().show(ctx, |ui| {
//!             ui.heading("Hello Elegant!");
//!             ui.add(ElegantButton::new("Primary Action").variant(Variant::Primary));
//!         });
//!     })
//! }
//! ```
//! 
//! For full component showcases, check out the `examples/` directory in the repository.
//!
//! Every leaf component implements the [`traits::Elegant`] trait.

mod accordion;
mod alert;
mod avatar;
mod badge;
mod button;
mod card;
mod dropdown;
mod inputs;
mod progress;
mod skeleton;
mod tabs;
mod taginput;
mod theme;
mod toast;
mod traits;

pub use accordion::ElegantAccordion;
pub use alert::Alert;
pub use avatar::Avatar;
pub use badge::ElegantBadge;
pub use button::ElegantButton;
pub use card::Card;
pub use dropdown::ElegantDropdown;
pub use inputs::InputUiExtensions;
pub use progress::Progress;
pub use skeleton::Skeleton;
pub use tabs::ElegantTabs;
pub use taginput::ElegantTagInput;
pub use theme::{
	ElegantFont, ElegantTheme, MonaspaceFont, ThemeMode, Variant, get_os_accent_color,
	is_system_dark_mode,
};
pub use toast::ElegantToast;
pub use traits::Elegant;

/// Re-export `egui_flex` so consumers don't need a separate direct dependency.
#[cfg(feature = "flex")]
pub use egui_flex;
