/// Implemented by all leaf elegant components (those that produce a single
/// [`egui::Response`] and implement [`egui::Widget`]).
///
/// Implementing this trait is all that's needed for a component to:
/// - Work in standard egui layouts via `ui.add(...)` (through `egui::Widget`)
/// - Work in `egui_flex` layouts via `flex.add(item(), ...)` when the `flex` feature is
///   enabled (through `crate::impl_flex_widget!` called in each module)
pub trait Elegant: egui::Widget {
	/// The natural minimum width this component should occupy in a flex layout.
	///
	/// This is a static, layout-level hint fed into `egui_flex::FlexItem`.
	/// Independent from inner rendering constraints like `ui.set_min_width()`.
	/// Returns `None` if the component is naturally content-sized (default).
	fn flex_default_min_width() -> Option<f32> {
		None
	}

	/// The natural minimum height this component should occupy in a flex layout.
	/// Returns `None` if the component is naturally content-sized (default).
	fn flex_default_min_height() -> Option<f32> {
		None
	}
}

/// Implements [`egui_flex::FlexWidget`] for a type that implements [`Elegant`].
///
/// Due to Rust orphan rules, a blanket `impl<W: Elegant> FlexWidget for W` is not
/// allowed (neither `FlexWidget` nor `W` is local). This macro generates an explicit
/// impl per component.
///
/// # Example
/// ```rust,ignore
/// impl_flex_widget!(ElegantButton<'a>, 'a);
/// impl_flex_widget!(Progress); // no lifetime
/// ```
#[cfg(feature = "flex")]
#[macro_export]
macro_rules! impl_flex_widget {
	// Variant for types with a lifetime parameter
	($type:ident<$lt:lifetime>) => {
		impl<$lt> $crate::egui_flex::FlexWidget for $type<$lt> {
			type Response = ::egui::Response;

			fn default_item() -> $crate::egui_flex::FlexItem<'static> {
				let mut item = $crate::egui_flex::FlexItem::new();
				if let Some(w) = <Self as $crate::Elegant>::flex_default_min_width() {
					item = item.min_width(w);
				}
				if let Some(h) = <Self as $crate::Elegant>::flex_default_min_height() {
					item = item.min_height(h);
				}
				item
			}

			fn flex_ui(
				self,
				item: $crate::egui_flex::FlexItem,
				flex: &mut $crate::egui_flex::FlexInstance,
			) -> ::egui::Response {
				flex.add_widget(item, self).inner
			}
		}
	};
	// Variant for types without a lifetime parameter
	($type:ident) => {
		impl $crate::egui_flex::FlexWidget for $type {
			type Response = ::egui::Response;

			fn default_item() -> $crate::egui_flex::FlexItem<'static> {
				let mut item = $crate::egui_flex::FlexItem::new();
				if let Some(w) = <Self as $crate::Elegant>::flex_default_min_width() {
					item = item.min_width(w);
				}
				if let Some(h) = <Self as $crate::Elegant>::flex_default_min_height() {
					item = item.min_height(h);
				}
				item
			}

			fn flex_ui(
				self,
				item: $crate::egui_flex::FlexItem,
				flex: &mut $crate::egui_flex::FlexInstance,
			) -> ::egui::Response {
				flex.add_widget(item, self).inner
			}
		}
	};
}

/// No-op stub when the `flex` feature is disabled, so call sites compile cleanly.
#[cfg(not(feature = "flex"))]
#[macro_export]
macro_rules! impl_flex_widget {
	($($tt:tt)*) => {};
}
