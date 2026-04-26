/// Generates a SAT catalog enum with:
/// - named variants mapping to SAT code strings
/// - Unknown(String) catch-all for forward compatibility
/// - Deserialize: string → variant or Unknown
/// - Serialize: variant → SAT code string
/// - Display: same as Serialize
macro_rules! catalog_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $( $variant:ident => $code:literal ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $name {
            $( $variant, )*
            Unknown(String),
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                struct V;
                impl<'de> serde::de::Visitor<'de> for V {
                    type Value = $name;
                    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(f, "a SAT catalog code string")
                    }
                    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<$name, E> {
                        Ok(match v {
                            $( $code => $name::$variant, )*
                            other => $name::Unknown(other.to_string()),
                        })
                    }
                }
                d.deserialize_str(V)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(match self {
                    $( $name::$variant => $code, )*
                    $name::Unknown(v) => v.as_str(),
                })
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $( $name::$variant => write!(f, $code), )*
                    $name::Unknown(v) => write!(f, "{}", v),
                }
            }
        }
    };
}

catalog_enum! {
    /// c_TipoDeComprobante — CFDI document type
    pub enum DocumentType {
        Income    => "I",
        Expense   => "E",
        Transfer  => "T",
        Payment   => "P",
        Payroll   => "N",
    }
}

catalog_enum! {
    /// c_RegimenFiscal — fiscal regime codes
    pub enum FiscalRegime {
        GeneralLegalEntities                 => "601",
        LegalEntitiesNonProfit               => "603",
        SalariedPersons                      => "605",
        RealEstateLeasing                    => "606",
        AlienationOrAcquisition              => "607",
        OtherIncome                          => "608",
        Consolidation                        => "609",
        ForeignResidents                     => "610",
        DividendIncome                       => "611",
        BusinessAndProfessional              => "612",
        InterestIncome                       => "614",
        PrizesIncome                         => "615",
        NoFiscalObligations                  => "616",
        CooperativeSocieties                 => "620",
        FiscalIncorporation                  => "621",
        AgriculturalActivities               => "622",
        OptionalGroupsOfCompanies            => "623",
        Coordinated                          => "624",
        DigitalPlatformIncome                => "625",
        SimplifiedTrustRegime                => "626",
    }
}

catalog_enum! {
    /// c_UsoCFDI — CFDI usage purpose codes
    pub enum CfdiUse {
        // General / investment
        AcquisitionOfGoods          => "G01",
        ReturnsDiscountsBonuses     => "G02",
        GeneralExpenses             => "G03",
        Construction                => "I01",
        TransportationEquipment     => "I02",
        OfficeEquipment             => "I03",
        ComputingEquipment          => "I04",
        CommunicationsEquipment     => "I05",
        ElectricalGenerationEquipment => "I06",
        OtherInvestmentConcepts     => "I07",
        RelatedInvestmentEquipment  => "I08",
        // Personal deductions
        MedicalExpenses             => "D01",
        FuneralExpenses             => "D02",
        Donations                   => "D03",
        MortgageInterest            => "D04",
        RetirementContributions     => "D05",
        MedicalInsurance            => "D06",
        EducationExpenses           => "D07",
        Housing                     => "D08",
        UnionContributions          => "D09",
        DependentCareContributions  => "D10",
        // Special
        NoFiscalEffects             => "S01",
        Payments                    => "CP01",
        Payroll                     => "CN01",
    }
}

catalog_enum! {
    /// c_FormaPago — payment form codes
    pub enum PaymentForm {
        Cash                        => "01",
        NominativeCheck             => "02",
        ElectronicTransfer          => "03",
        CreditCard                  => "04",
        ElectronicWallet            => "05",
        EMoney                      => "06",
        FoodVouchers                => "08",
        PaymentByAssetDelivery      => "12",
        PaymentBySubrogation        => "13",
        PaymentByConsignment        => "14",
        Remission                   => "15",
        Compensation                => "17",
        Novation                    => "23",
        Confusion                   => "24",
        DebtRemission               => "25",
        Prescription                => "26",
        ToCreditorSatisfaction      => "27",
        DebitCard                   => "28",
        ServiceCard                 => "29",
        AdvanceApplication          => "30",
        PaymentIntermediary         => "31",
        Cryptocurrency              => "32",
        ToBeDefined                 => "99",
    }
}

catalog_enum! {
    /// c_MetodoPago — payment method codes
    pub enum PaymentMethod {
        SinglePayment   => "PUE",
        Installments    => "PPD",
    }
}

catalog_enum! {
    /// c_Impuesto — tax type codes
    pub enum TaxType {
        Isr  => "001",
        Iva  => "002",
        Ieps => "003",
    }
}

catalog_enum! {
    /// c_TipoFactor — tax calculation factor type
    pub enum TaxFactor {
        Rate   => "Tasa",
        Amount => "Cuota",
        Exempt => "Exento",
    }
}

catalog_enum! {
    /// c_ObjetoImp — tax object classification
    pub enum TaxObject {
        NotSubject          => "01",
        Subject             => "02",
        SubjectNoBreakdown  => "03",
        SubjectNoTax        => "04",
        SubjectPartial      => "05",
        SubjectNoVatTransfer => "06",
    }
}

catalog_enum! {
    /// c_Exportacion — export type
    pub enum ExportType {
        NotApplicable    => "01",
        Definitive       => "02",
        Temporary        => "03",
        DefinitiveOther  => "04",
    }
}

catalog_enum! {
    /// c_TipoRelacion — CFDI relationship type between linked documents
    pub enum RelationType {
        CreditNote              => "01",
        DebitNote               => "02",
        MerchandiseReturn       => "03",
        Substitution            => "04",
        TransferOfPreviousGoods => "05",
        InvoiceFromTransfer     => "06",
        AdvanceApplication      => "07",
    }
}

catalog_enum! {
    /// c_Moneda — currency codes (ISO 4217 subset used by SAT + common ones)
    pub enum Currency {
        Mxn => "MXN",
        Usd => "USD",
        Eur => "EUR",
        Cad => "CAD",
        Gbp => "GBP",
        Jpy => "JPY",
        Chf => "CHF",
        Aud => "AUD",
        Cny => "CNY",
        Brl => "BRL",
        Cop => "COP",
        Ars => "ARS",
        Clp => "CLP",
        Pen => "PEN",
        Xxx => "XXX",
    }
}

catalog_enum! {
    /// c_PeriodicidadPago (nomina) — payroll payment periodicity
    pub enum PayrollPeriodicity {
        Daily       => "01",
        Weekly      => "02",
        Biweekly    => "03",
        Monthly     => "04",
        Bimonthly   => "05",
        Semiannual  => "06",
        ByUnitWork  => "07",
        ByCommission => "08",
        FixedPrice  => "09",
        TenDay      => "10",
        Other       => "99",
    }
}

catalog_enum! {
    /// c_TipoContrato — employment contract type
    pub enum ContractType {
        Indefinite          => "01",
        SpecificWork        => "02",
        FixedTerm           => "03",
        Seasonal            => "04",
        TrialPeriod         => "05",
        InitialTraining     => "06",
        HourlyPayment       => "07",
        CommissionBased     => "08",
    }
}

catalog_enum! {
    /// c_TipoRegimen (nomina) — payroll regime type
    pub enum PaymentRegime {
        SalariesAndWages             => "02",
        Retirees                     => "03",
        Pensioners                   => "04",
        CooperativeMembers           => "05",
        CivilAssociationMembers      => "06",
        Commissioners                => "07",
        BoardMembers                 => "08",
        ProfessionalServices         => "09",
        ArbitratorsNotariesMediators => "10",
        ConstructionContractors      => "11",
        SeveranceIndemnification     => "13",
    }
}

catalog_enum! {
    /// c_Periodicidad — global invoice periodicity
    pub enum InvoicePeriodicity {
        Daily       => "01",
        Weekly      => "02",
        Biweekly    => "03",
        Monthly     => "04",
        Bimonthly   => "05",
    }
}
