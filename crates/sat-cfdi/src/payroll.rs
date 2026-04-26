use crate::catalogs::{ContractType, PaymentRegime, PayrollPeriodicity};
use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_si_no<'de, D: Deserializer<'de>>(d: D) -> Result<Option<bool>, D::Error> {
    let s: Option<String> = Option::deserialize(d)?;
    Ok(s.as_deref().map(|v| v == "Sí"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollComplement {
    #[serde(rename(deserialize = "@Version"))]
    pub version: String,
    #[serde(rename(deserialize = "@TipoNomina"))]
    pub payroll_type: String,
    #[serde(rename(deserialize = "@FechaPago"))]
    pub payment_date: String,
    #[serde(rename(deserialize = "@FechaInicialPago"))]
    pub period_start: String,
    #[serde(rename(deserialize = "@FechaFinalPago"))]
    pub period_end: String,
    #[serde(rename(deserialize = "@NumDiasPagados"))]
    pub days_paid: String,
    #[serde(rename(deserialize = "@TotalPercepciones"), default)]
    pub total_earnings: Option<String>,
    #[serde(rename(deserialize = "@TotalDeducciones"), default)]
    pub total_deductions: Option<String>,
    #[serde(rename(deserialize = "@TotalOtrosPagos"), default)]
    pub total_other_payments: Option<String>,

    #[serde(rename(deserialize = "Emisor"), default)]
    pub employer: Option<PayrollEmployer>,
    #[serde(rename(deserialize = "Receptor"))]
    pub employee: PayrollEmployee,
    #[serde(rename(deserialize = "Percepciones"), default)]
    pub earnings: Option<Earnings>,
    #[serde(rename(deserialize = "Deducciones"), default)]
    pub deductions: Option<Deductions>,
    #[serde(rename(deserialize = "OtrosPagos"), default)]
    pub other_payments: Option<OtherPayments>,
    #[serde(rename(deserialize = "Incapacidades"), default)]
    pub disabilities: Option<Disabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollEmployer {
    #[serde(rename(deserialize = "@Curp"), default)]
    pub curp: Option<String>,
    #[serde(rename(deserialize = "@RegistroPatronal"), default)]
    pub employer_registration: Option<String>,
    #[serde(rename(deserialize = "@RfcPatronOrigen"), default)]
    pub origin_employer_tax_id: Option<String>,
    #[serde(rename(deserialize = "EntidadSNCF"), default)]
    pub snfc_entity: Option<SncfEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SncfEntity {
    #[serde(rename(deserialize = "@OrigenRecurso"))]
    pub resource_origin: String,
    #[serde(rename(deserialize = "@MontoRecursoPropio"), default)]
    pub own_resource_amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollEmployee {
    #[serde(rename(deserialize = "@Curp"))]
    pub curp: String,
    #[serde(rename(deserialize = "@NumSeguridadSocial"), default)]
    pub social_security_number: Option<String>,
    #[serde(rename(deserialize = "@FechaInicioRelLaboral"), default)]
    pub employment_start_date: Option<String>,
    #[serde(rename(deserialize = "@Antigüedad"), default)]
    pub seniority: Option<String>,
    #[serde(rename(deserialize = "@TipoContrato"))]
    pub contract_type: ContractType,
    #[serde(
        rename(deserialize = "@Sindicalizado"),
        deserialize_with = "deserialize_si_no",
        default
    )]
    pub unionized: Option<bool>,
    #[serde(rename(deserialize = "@TipoJornada"), default)]
    pub workday_type: Option<String>,
    #[serde(rename(deserialize = "@TipoRegimen"))]
    pub payment_regime: PaymentRegime,
    #[serde(rename(deserialize = "@NumEmpleado"))]
    pub employee_number: String,
    #[serde(rename(deserialize = "@Departamento"), default)]
    pub department: Option<String>,
    #[serde(rename(deserialize = "@Puesto"), default)]
    pub position: Option<String>,
    #[serde(rename(deserialize = "@RiesgoPuesto"), default)]
    pub job_risk: Option<String>,
    #[serde(rename(deserialize = "@PeriodicidadPago"))]
    pub payment_periodicity: PayrollPeriodicity,
    #[serde(rename(deserialize = "@Banco"), default)]
    pub bank: Option<String>,
    #[serde(rename(deserialize = "@CuentaBancaria"), default)]
    pub bank_account: Option<String>,
    #[serde(rename(deserialize = "@SalarioBaseCotApor"), default)]
    pub base_salary: Option<String>,
    #[serde(rename(deserialize = "@SalarioDiarioIntegrado"), default)]
    pub integrated_daily_salary: Option<String>,
    #[serde(rename(deserialize = "@ClaveEntFed"))]
    pub state_key: String,

    #[serde(rename(deserialize = "SubContratacion"), default)]
    pub subcontracting: Vec<Subcontracting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subcontracting {
    #[serde(rename(deserialize = "@RfcLabora"))]
    pub employer_tax_id: String,
    #[serde(rename(deserialize = "@PorcentajeTiempo"))]
    pub time_percentage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Earnings {
    #[serde(rename(deserialize = "@TotalSueldos"), default)]
    pub total_salaries: Option<String>,
    #[serde(rename(deserialize = "@TotalSeparacionIndemnizacion"), default)]
    pub total_separation: Option<String>,
    #[serde(rename(deserialize = "@TotalJubilacionPensionRetiro"), default)]
    pub total_retirement: Option<String>,
    #[serde(rename(deserialize = "@TotalGravado"))]
    pub total_taxable: String,
    #[serde(rename(deserialize = "@TotalExento"))]
    pub total_exempt: String,

    #[serde(rename(deserialize = "Percepcion"), default)]
    pub items: Vec<Earning>,
    #[serde(rename(deserialize = "JubilacionPensionRetiro"), default)]
    pub retirement_payment: Option<RetirementPayment>,
    #[serde(rename(deserialize = "SeparacionIndemnizacion"), default)]
    pub separation_payment: Option<SeparationPayment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Earning {
    #[serde(rename(deserialize = "@TipoPercepcion"))]
    pub earning_type: String,
    #[serde(rename(deserialize = "@Clave"))]
    pub code: String,
    #[serde(rename(deserialize = "@Concepto"))]
    pub concept: String,
    #[serde(rename(deserialize = "@ImporteGravado"))]
    pub taxable_amount: String,
    #[serde(rename(deserialize = "@ImporteExento"))]
    pub exempt_amount: String,

    #[serde(rename(deserialize = "AccionesOTitulos"), default)]
    pub stock_options: Option<StockOptions>,
    #[serde(rename(deserialize = "HorasExtra"), default)]
    pub overtime: Vec<Overtime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockOptions {
    #[serde(rename(deserialize = "@ValorMercado"))]
    pub market_value: String,
    #[serde(rename(deserialize = "@PrecioAlOtorgarse"))]
    pub grant_price: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Overtime {
    #[serde(rename(deserialize = "@Dias"))]
    pub days: String,
    #[serde(rename(deserialize = "@TipoHoras"))]
    pub hours_type: String,
    #[serde(rename(deserialize = "@HorasExtra"))]
    pub extra_hours: String,
    #[serde(rename(deserialize = "@ImportePagado"))]
    pub paid_amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetirementPayment {
    #[serde(rename(deserialize = "@TotalUnaExhibicion"), default)]
    pub total_lump_sum: Option<String>,
    #[serde(rename(deserialize = "@TotalParcialidad"), default)]
    pub total_installment: Option<String>,
    #[serde(rename(deserialize = "@MontoDiario"), default)]
    pub daily_amount: Option<String>,
    #[serde(rename(deserialize = "@IngresoAcumulable"))]
    pub cumulative_income: String,
    #[serde(rename(deserialize = "@IngresoNoAcumulable"))]
    pub non_cumulative_income: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeparationPayment {
    #[serde(rename(deserialize = "@TotalPagado"))]
    pub total_paid: String,
    #[serde(rename(deserialize = "@NumAñosServicio"))]
    pub years_of_service: String,
    #[serde(rename(deserialize = "@UltimoSueldoMensOrd"))]
    pub last_ordinary_monthly_salary: String,
    #[serde(rename(deserialize = "@IngresoAcumulable"))]
    pub cumulative_income: String,
    #[serde(rename(deserialize = "@IngresoNoAcumulable"))]
    pub non_cumulative_income: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deductions {
    #[serde(rename(deserialize = "@TotalOtrasDeducciones"), default)]
    pub total_other_deductions: Option<String>,
    #[serde(rename(deserialize = "@TotalImpuestosRetenidos"), default)]
    pub total_taxes_withheld: Option<String>,
    #[serde(rename(deserialize = "Deduccion"), default)]
    pub items: Vec<Deduction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deduction {
    #[serde(rename(deserialize = "@TipoDeduccion"))]
    pub deduction_type: String,
    #[serde(rename(deserialize = "@Clave"))]
    pub code: String,
    #[serde(rename(deserialize = "@Concepto"))]
    pub concept: String,
    #[serde(rename(deserialize = "@Importe"))]
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherPayments {
    #[serde(rename(deserialize = "OtroPago"), default)]
    pub items: Vec<OtherPayment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherPayment {
    #[serde(rename(deserialize = "@TipoOtroPago"))]
    pub payment_type: String,
    #[serde(rename(deserialize = "@Clave"))]
    pub code: String,
    #[serde(rename(deserialize = "@Concepto"))]
    pub concept: String,
    #[serde(rename(deserialize = "@Importe"))]
    pub amount: String,

    #[serde(rename(deserialize = "SubsidioAlEmpleo"), default)]
    pub employment_subsidy: Option<EmploymentSubsidy>,
    #[serde(rename(deserialize = "CompensacionSaldosAFavor"), default)]
    pub balance_compensation: Option<BalanceCompensation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentSubsidy {
    #[serde(rename(deserialize = "@SubsidioCausado"))]
    pub subsidy_caused: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceCompensation {
    #[serde(rename(deserialize = "@SaldoAFavor"))]
    pub balance_in_favor: String,
    #[serde(rename(deserialize = "@Año"))]
    pub year: String,
    #[serde(rename(deserialize = "@RemanenteSalFav"))]
    pub remaining_balance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disabilities {
    #[serde(rename(deserialize = "Incapacidad"), default)]
    pub items: Vec<Disability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disability {
    #[serde(rename(deserialize = "@DiasIncapacidad"))]
    pub days: String,
    #[serde(rename(deserialize = "@TipoIncapacidad"))]
    pub disability_type: String,
    #[serde(rename(deserialize = "@ImporteMonetario"), default)]
    pub monetary_amount: Option<String>,
}
