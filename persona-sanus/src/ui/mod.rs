#[cfg(debug_assertions)]
#[allow(dead_code)]
mod billing;
#[allow(dead_code)]
mod brand;
#[allow(dead_code)]
mod button;
#[allow(dead_code)]
mod domain;
#[allow(dead_code)]
mod feedback;
#[allow(dead_code)]
mod form;
mod format;
#[allow(dead_code)]
mod icon;
#[allow(dead_code)]
mod layout;
#[allow(dead_code)]
mod marketing;

#[cfg(debug_assertions)]
#[allow(unused_imports)]
pub use billing::{
    format_usd_cents, BillingBanner, BillingStatusCard, BillingTransaction, PaymentMethodCard,
    PlanCard, TransactionTable,
};
#[allow(unused_imports)]
pub use brand::{LogoLockup, TrustLogo, TrustLogoStrip, TrustSecurityRow};
#[allow(unused_imports)]
pub use button::{Button, ButtonLink, ButtonVariant, LinkAction, NavLink};
#[allow(unused_imports)]
pub use domain::{
    AccountStatusBadge, DeviceStatusBadge, ImageLibrary, LlmExplainLink, PrescriptionStatusBadge,
    QuotaMeter, RxBlockIndicator, SyncStatus, SyncStatusIndicator, TemplateStatusBadge,
    VerificationCodeForm, VerificationResultCard, VerificationSeal,
};
#[allow(unused_imports)]
pub use feedback::{
    Badge, BadgeTone, ConfirmPanel, DataTable, EmptyState, ErrorText, HintText, ListRow,
    LoadingState, Notice, NoticeTone, PillBadge,
};
#[allow(unused_imports)]
pub use form::{CheckboxField, FileField, SearchField, SelectField, TextAreaField, TextField};
#[allow(unused_imports)]
pub use format::{
    device_name_label, format_date, format_timestamp, last_seen_label, optional_date,
    platform_label, prescription_status_label, role_label, sex_label, status_label,
    template_owner_label, template_status_label, version_status_label,
};
#[allow(unused_imports)]
pub use icon::{Icon, IconName};
#[allow(unused_imports)]
pub use layout::{
    ActionRow, Card, DetailGrid, DetailItem, FilterBar, FormColumns, FormGrid, PageHeader,
    PageShell, SectionBand, SectionCard, SurfaceTone,
};
#[allow(unused_imports)]
pub use marketing::{
    A4Mock, A4MockKind, A4PreviewCard, AccentTone, Hero, NumberedStepCard, SectionHeader,
    VerificationTag,
};
