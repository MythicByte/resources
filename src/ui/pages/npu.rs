use adw::{prelude::*, subclass::prelude::*};
use gtk::glib::{self};

use crate::config::PROFILE;
use crate::i18n::{i18n, i18n_f};
use crate::utils::npu::{Npu, NpuData};
use crate::utils::units::{convert_frequency, convert_power, convert_storage, convert_temperature};
use crate::utils::FiniteOr;

pub const TAB_ID_PREFIX: &str = "npu";

mod imp {
    use std::cell::{Cell, RefCell};

    use crate::ui::{pages::NPU_PRIMARY_ORD, widgets::graph_box::ResGraphBox};

    use super::*;

    use gtk::{
        gio::{Icon, ThemedIcon},
        glib::{ParamSpec, Properties, Value},
        CompositeTemplate,
    };

    #[derive(CompositeTemplate, Properties)]
    #[template(resource = "/net/nokyan/Resources/ui/pages/npu.ui")]
    #[properties(wrapper_type = super::ResNPU)]
    pub struct ResNPU {
        #[template_child]
        pub npu_usage: TemplateChild<ResGraphBox>,
        #[template_child]
        pub memory_usage: TemplateChild<ResGraphBox>,
        #[template_child]
        pub temperature: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub power_usage: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub npu_clockspeed: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub memory_clockspeed: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub manufacturer: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub pci_slot: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub driver_used: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub max_power_cap: TemplateChild<adw::ActionRow>,

        #[property(get)]
        uses_progress_bar: Cell<bool>,

        #[property(get)]
        main_graph_color: glib::Bytes,

        #[property(get)]
        icon: RefCell<Icon>,

        #[property(get, set)]
        usage: Cell<f64>,

        #[property(get = Self::tab_name, set = Self::set_tab_name, type = glib::GString)]
        tab_name: Cell<glib::GString>,

        #[property(get = Self::tab_detail, set = Self::set_tab_detail, type = glib::GString)]
        tab_detail_string: Cell<glib::GString>,

        #[property(get = Self::tab_usage_string, set = Self::set_tab_usage_string, type = glib::GString)]
        tab_usage_string: Cell<glib::GString>,

        #[property(get = Self::tab_id, set = Self::set_tab_id, type = glib::GString)]
        tab_id: Cell<glib::GString>,

        #[property(get)]
        graph_locked_max_y: Cell<bool>,

        #[property(get)]
        primary_ord: Cell<u32>,

        #[property(get, set)]
        secondary_ord: Cell<u32>,
    }

    impl ResNPU {
        pub fn tab_name(&self) -> glib::GString {
            let tab_name = self.tab_name.take();
            let result = tab_name.clone();
            self.tab_name.set(tab_name);
            result
        }

        pub fn set_tab_name(&self, tab_name: &str) {
            self.tab_name.set(glib::GString::from(tab_name));
        }

        pub fn tab_detail(&self) -> glib::GString {
            let detail = self.tab_detail_string.take();
            let result = detail.clone();
            self.tab_detail_string.set(detail);
            result
        }

        pub fn set_tab_detail(&self, detail: &str) {
            self.tab_detail_string.set(glib::GString::from(detail));
        }

        pub fn tab_usage_string(&self) -> glib::GString {
            let tab_usage_string = self.tab_usage_string.take();
            let result = tab_usage_string.clone();
            self.tab_usage_string.set(tab_usage_string);
            result
        }

        pub fn set_tab_usage_string(&self, tab_usage_string: &str) {
            self.tab_usage_string
                .set(glib::GString::from(tab_usage_string));
        }

        pub fn tab_id(&self) -> glib::GString {
            let tab_id = self.tab_id.take();
            let result = tab_id.clone();
            self.tab_id.set(tab_id);
            result
        }

        pub fn set_tab_id(&self, tab_id: &str) {
            self.tab_id.set(glib::GString::from(tab_id));
        }
    }

    impl Default for ResNPU {
        fn default() -> Self {
            Self {
                npu_usage: Default::default(),
                memory_usage: Default::default(),
                temperature: Default::default(),
                power_usage: Default::default(),
                npu_clockspeed: Default::default(),
                memory_clockspeed: Default::default(),
                manufacturer: Default::default(),
                pci_slot: Default::default(),
                driver_used: Default::default(),
                max_power_cap: Default::default(),
                uses_progress_bar: Cell::new(true),
                main_graph_color: glib::Bytes::from_static(&super::ResNPU::MAIN_GRAPH_COLOR),
                icon: RefCell::new(ThemedIcon::new("npu-symbolic").into()),
                usage: Default::default(),
                tab_name: Cell::new(glib::GString::from(i18n("NPU"))),
                tab_detail_string: Cell::new(glib::GString::new()),
                tab_usage_string: Cell::new(glib::GString::new()),
                tab_id: Cell::new(glib::GString::new()),
                graph_locked_max_y: Cell::new(true),
                primary_ord: Cell::new(NPU_PRIMARY_ORD),
                secondary_ord: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ResNPU {
        const NAME: &'static str = "ResNPU";
        type Type = super::ResNPU;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ResNPU {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
        }

        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }
    }

    impl WidgetImpl for ResNPU {}
    impl BinImpl for ResNPU {}
}

glib::wrapper! {
    pub struct ResNPU(ObjectSubclass<imp::ResNPU>)
        @extends gtk::Widget, adw::Bin;
}

impl Default for ResNPU {
    fn default() -> Self {
        Self::new()
    }
}

impl ResNPU {
    const MAIN_GRAPH_COLOR: [u8; 3] = [0xb5, 0x27, 0xe3];

    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }

    pub fn init(&self, npu: &Npu, secondary_ord: u32) {
        self.set_secondary_ord(secondary_ord);
        self.setup_widgets(npu);
    }

    pub fn setup_widgets(&self, npu: &Npu) {
        let imp = self.imp();

        let tab_id = format!("{}-{}", TAB_ID_PREFIX, &npu.pci_slot().to_string());
        imp.set_tab_id(&tab_id);

        imp.npu_usage.set_title_label(&i18n("Total Usage"));
        imp.npu_usage.graph().set_graph_color(
            Self::MAIN_GRAPH_COLOR[0],
            Self::MAIN_GRAPH_COLOR[1],
            Self::MAIN_GRAPH_COLOR[2],
        );

        imp.memory_usage.set_title_label(&i18n("Memory Usage"));
        imp.memory_usage.graph().set_graph_color(0x9e, 0xc, 0xcc);

        imp.manufacturer.set_subtitle(
            &npu.get_vendor()
                .map_or_else(|_| i18n("N/A"), |vendor| vendor.name().to_string()),
        );

        imp.pci_slot.set_subtitle(&npu.pci_slot().to_string());

        imp.driver_used.set_subtitle(&npu.driver());

        if let Ok(model_name) = npu.name() {
            imp.set_tab_detail(&model_name);
        }
    }

    pub fn refresh_page(&self, npu_data: &NpuData) {
        let imp = self.imp();

        let NpuData {
            pci_slot: _,
            usage_fraction,
            total_vram,
            used_vram,
            clock_speed,
            vram_speed,
            temp,
            power_usage,
            power_cap,
            power_cap_max,
        } = npu_data;

        let mut usage_percentage_string = usage_fraction.map_or_else(
            || i18n("N/A"),
            |fraction| format!("{} %", (fraction * 100.0).round()),
        );

        imp.npu_usage.set_subtitle(&usage_percentage_string);
        imp.npu_usage
            .graph()
            .push_data_point(usage_fraction.unwrap_or(0.0));
        imp.npu_usage.graph().set_visible(usage_fraction.is_some());

        let used_vram_fraction =
            if let (Some(total_vram), Some(used_vram)) = (total_vram, used_vram) {
                Some((*used_vram as f64 / *total_vram as f64).finite_or_default())
            } else {
                None
            };

        let vram_percentage_string = used_vram_fraction.as_ref().map_or_else(
            || i18n("N/A"),
            |fraction| format!("{} %", (fraction * 100.0).round()),
        );

        let vram_subtitle = if let (Some(total_vram), Some(used_vram)) = (total_vram, used_vram) {
            format!(
                "{} / {} · {}",
                convert_storage(*used_vram as f64, false),
                convert_storage(*total_vram as f64, false),
                vram_percentage_string
            )
        } else {
            i18n("N/A")
        };

        imp.memory_usage.set_subtitle(&vram_subtitle);
        imp.memory_usage
            .graph()
            .push_data_point(used_vram_fraction.unwrap_or(0.0));
        imp.memory_usage
            .graph()
            .set_visible(used_vram_fraction.is_some());

        let temperature_string = temp.map(convert_temperature);

        imp.temperature
            .set_subtitle(&temperature_string.clone().unwrap_or_else(|| i18n("N/A")));

        let mut power_string = power_usage.map_or_else(|| i18n("N/A"), convert_power);

        if let Some(power_cap) = power_cap {
            power_string.push_str(&format!(" / {}", convert_power(*power_cap)));
        }

        imp.power_usage.set_subtitle(&power_string);

        if let Some(npu_clockspeed) = clock_speed {
            imp.npu_clockspeed
                .set_subtitle(&convert_frequency(*npu_clockspeed));
        } else {
            imp.npu_clockspeed.set_subtitle(&i18n("N/A"));
        }

        if let Some(vram_clockspeed) = vram_speed {
            imp.memory_clockspeed
                .set_subtitle(&convert_frequency(*vram_clockspeed));
        } else {
            imp.memory_clockspeed.set_subtitle(&i18n("N/A"));
        }

        imp.max_power_cap
            .set_subtitle(&power_cap_max.map_or_else(|| i18n("N/A"), convert_power));

        self.set_property("usage", usage_fraction.unwrap_or(0.0));

        if used_vram_fraction.is_some() {
            usage_percentage_string.push_str(" · ");
            // Translators: This will be displayed in the sidebar, please try to keep your translation as short as (or even
            // shorter than) 'Memory'
            usage_percentage_string.push_str(&i18n_f("Memory: {}", &[&vram_percentage_string]));
        }

        if let Some(temperature_string) = temperature_string {
            usage_percentage_string.push_str(" · ");
            usage_percentage_string.push_str(&temperature_string);
        }

        self.set_property("tab_usage_string", &usage_percentage_string);
    }
}
