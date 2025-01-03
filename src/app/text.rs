use crate::special::composition::{
    Composition, Kind, MC, NC, PMC, PNC, PSC, PTC, PUC, SC, SMC, SNC, SSC, STC, SUC, TC, UC,
};

// Text
pub trait Text {
    fn text(&self) -> &'static str;

    fn hover_text(&self) -> &'static str;
}

impl Text for Composition {
    fn text(&self) -> &'static str {
        match *self {
            NC => "NC",
            PNC => "PNC",
            SNC => "SNC",

            MC => "MC",
            PMC => "PMC",
            SMC => "SMC",

            UC => "UC",
            PUC => "PUC",
            SUC => "SUC",

            TC => "TC",
            PTC => "PTC",
            STC => "STC",

            SC => "SC",
            PSC => "PSC",
            SSC => "SSC",
        }
    }

    fn hover_text(&self) -> &'static str {
        match *self {
            NC => "Equivalent carbon number composition",
            PNC => "Positional equivalent carbon number composition",
            SNC => "Stereo equivalent carbon number composition",

            MC => "Mass composition",
            PMC => "Positional mass composition",
            SMC => "Stereo mass composition",

            UC => "Unsaturation composition",
            PUC => "Positional unsaturation composition",
            SUC => "Stereo unsaturation composition",

            TC => "Type composition",
            PTC => "Positional type composition",
            STC => "Stereo type composition",

            SC => "Species composition",
            PSC => "Positional species composition",
            SSC => "Stereo species composition",
        }
    }
}

impl Text for Kind {
    fn text(&self) -> &'static str {
        match self {
            Self::Ecn => "Equivalent carbon number",
            Self::Mass => "Mass",
            Self::Species => "Species",
            Self::Type => "Type",
            Self::Unsaturation => "Unsaturation",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Ecn => "ECN",
            Self::Mass => "M",
            Self::Species => "S",
            Self::Type => "T",
            Self::Unsaturation => "U",
        }
    }
}
