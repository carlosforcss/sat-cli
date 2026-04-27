use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_si_no<'de, D: Deserializer<'de>>(d: D) -> Result<Option<bool>, D::Error> {
    let s: Option<String> = Option::deserialize(d)?;
    Ok(s.as_deref().map(|v| v == "Sí"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreightTransportComplement {
    #[serde(rename(deserialize = "@Version"))]
    pub version: String,
    #[serde(rename(deserialize = "@IdCCP"), default)]
    pub id_ccp: Option<String>,
    #[serde(
        rename(deserialize = "@TranspInternac"),
        deserialize_with = "deserialize_si_no",
        default
    )]
    pub is_international: Option<bool>,
    #[serde(rename(deserialize = "@EntradaSalidaMerc"), default)]
    pub goods_entry_exit: Option<String>,
    #[serde(rename(deserialize = "@PaisOrigenDestino"), default)]
    pub origin_destination_country: Option<String>,
    #[serde(rename(deserialize = "@ViaEntradaSalida"), default)]
    pub entry_exit_route: Option<String>,
    #[serde(rename(deserialize = "@TotalDistRec"), default)]
    pub total_distance: Option<String>,
    #[serde(rename(deserialize = "@RegistroISTMO"), default)]
    pub istmo_registration: Option<String>,
    #[serde(rename(deserialize = "@UbicacionPoloOrigen"), default)]
    pub pole_origin_location: Option<String>,
    #[serde(rename(deserialize = "@UbicacionPoloDestino"), default)]
    pub pole_destination_location: Option<String>,

    #[serde(rename(deserialize = "RegimenesAduaneros"), default)]
    pub customs_regimes: Option<CustomsRegimes>,
    #[serde(rename(deserialize = "Ubicaciones"))]
    pub locations: Locations,
    #[serde(rename(deserialize = "Mercancias"))]
    pub goods: Goods,
    #[serde(rename(deserialize = "FiguraTransporte"), default)]
    pub transport_figures: Option<TransportFigures>,
    #[serde(rename(deserialize = "Autotransporte"), default)]
    pub road_transport: Option<RoadTransport>,
    #[serde(rename(deserialize = "TransporteMaritimo"), default)]
    pub maritime_transport: Option<MaritimeTransportShell>,
    #[serde(rename(deserialize = "TransporteAereo"), default)]
    pub air_transport: Option<AirTransportShell>,
    #[serde(rename(deserialize = "TransporteFerroviario"), default)]
    pub rail_transport: Option<RailTransportShell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomsRegimes {
    #[serde(rename(deserialize = "RegimenAduaneroCCP"), default)]
    pub items: Vec<CustomsRegime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomsRegime {
    #[serde(rename(deserialize = "@RegimenAduanero"))]
    pub regime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Locations {
    #[serde(rename(deserialize = "Ubicacion"), default)]
    pub items: Vec<Location>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    #[serde(rename(deserialize = "@TipoUbicacion"))]
    pub location_type: String,
    #[serde(rename(deserialize = "@IDUbicacion"), default)]
    pub location_id: Option<String>,
    #[serde(rename(deserialize = "@RFCRemitenteDestinatario"))]
    pub shipper_consignee_tax_id: String,
    #[serde(rename(deserialize = "@NombreRemitenteDestinatario"), default)]
    pub shipper_consignee_name: Option<String>,
    #[serde(rename(deserialize = "@NumRegIdTrib"), default)]
    pub foreign_tax_id: Option<String>,
    #[serde(rename(deserialize = "@ResidenciaFiscal"), default)]
    pub tax_residence: Option<String>,
    #[serde(rename(deserialize = "@NumEstacion"), default)]
    pub station_number: Option<String>,
    #[serde(rename(deserialize = "@NombreEstacion"), default)]
    pub station_name: Option<String>,
    #[serde(rename(deserialize = "@NavegacionTrafico"), default)]
    pub navigation_traffic: Option<String>,
    #[serde(rename(deserialize = "@FechaHoraSalidaLlegada"))]
    pub departure_arrival_datetime: String,
    #[serde(rename(deserialize = "@TipoEstacion"), default)]
    pub station_type: Option<String>,
    #[serde(rename(deserialize = "@DistanciaRecorrida"), default)]
    pub distance_traveled: Option<String>,

    #[serde(rename(deserialize = "Domicilio"), default)]
    pub address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    #[serde(rename(deserialize = "@Calle"), default)]
    pub street: Option<String>,
    #[serde(rename(deserialize = "@NumeroExterior"), default)]
    pub exterior_number: Option<String>,
    #[serde(rename(deserialize = "@NumeroInterior"), default)]
    pub interior_number: Option<String>,
    #[serde(rename(deserialize = "@Colonia"), default)]
    pub neighborhood: Option<String>,
    #[serde(rename(deserialize = "@Localidad"), default)]
    pub locality: Option<String>,
    #[serde(rename(deserialize = "@Referencia"), default)]
    pub reference: Option<String>,
    #[serde(rename(deserialize = "@Municipio"), default)]
    pub municipality: Option<String>,
    #[serde(rename(deserialize = "@Estado"))]
    pub state: String,
    #[serde(rename(deserialize = "@Pais"))]
    pub country: String,
    #[serde(rename(deserialize = "@CodigoPostal"))]
    pub postal_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goods {
    #[serde(rename(deserialize = "@PesoBrutoTotal"), default)]
    pub total_gross_weight: Option<String>,
    #[serde(rename(deserialize = "@UnidadPeso"), default)]
    pub weight_unit: Option<String>,
    #[serde(rename(deserialize = "@PesoNetoTotal"), default)]
    pub total_net_weight: Option<String>,
    #[serde(rename(deserialize = "@NumTotalMercancias"))]
    pub total_items: String,
    #[serde(rename(deserialize = "@CargoPorTasacion"), default)]
    pub assessment_charge: Option<String>,

    #[serde(rename(deserialize = "Mercancia"), default)]
    pub items: Vec<Merchandise>,
    #[serde(rename(deserialize = "AutotransporteFederal"), default)]
    pub federal_road_transport: Option<FederalRoadTransport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Merchandise {
    #[serde(rename(deserialize = "@BienesTransp"))]
    pub goods_key: String,
    #[serde(rename(deserialize = "@ClaveSTCC"), default)]
    pub stcc_key: Option<String>,
    #[serde(rename(deserialize = "@Descripcion"))]
    pub description: String,
    #[serde(rename(deserialize = "@Cantidad"))]
    pub quantity: String,
    #[serde(rename(deserialize = "@ClaveUnidad"))]
    pub unit_key: String,
    #[serde(rename(deserialize = "@Unidad"), default)]
    pub unit: Option<String>,
    #[serde(rename(deserialize = "@Dimensiones"), default)]
    pub dimensions: Option<String>,
    #[serde(rename(deserialize = "@MaterialPeligroso"), default)]
    pub hazardous_material: Option<String>,
    #[serde(rename(deserialize = "@CveMaterialPeligroso"), default)]
    pub hazardous_material_key: Option<String>,
    #[serde(rename(deserialize = "@Embalaje"), default)]
    pub packaging: Option<String>,
    #[serde(rename(deserialize = "@DescripEmbalaje"), default)]
    pub packaging_description: Option<String>,
    #[serde(rename(deserialize = "@PesoEnKg"))]
    pub weight_kg: String,
    #[serde(rename(deserialize = "@ValorMercancia"), default)]
    pub merchandise_value: Option<String>,
    #[serde(rename(deserialize = "@Moneda"), default)]
    pub currency: Option<String>,
    #[serde(rename(deserialize = "@FraccionArancelaria"), default)]
    pub tariff_fraction: Option<String>,
    #[serde(rename(deserialize = "@UUIDComercioExt"), default)]
    pub foreign_trade_uuid: Option<String>,
    #[serde(rename(deserialize = "@TipoMateria"), default)]
    pub material_type: Option<String>,
    #[serde(rename(deserialize = "@DescripcionMateria"), default)]
    pub material_description: Option<String>,

    #[serde(rename(deserialize = "DocumentacionAduanera"), default)]
    pub customs_docs: Vec<CustomsDoc>,
    #[serde(rename(deserialize = "GuiasIdentificacion"), default)]
    pub tracking_numbers: Vec<TrackingId>,
    #[serde(rename(deserialize = "CantidadTransporta"), default)]
    pub quantity_transport: Vec<QuantityTransport>,
    #[serde(rename(deserialize = "DetalleMercancia"), default)]
    pub detail: Option<MerchandiseDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomsDoc {
    #[serde(rename(deserialize = "@TipoDocumento"))]
    pub document_type: String,
    #[serde(rename(deserialize = "@NumPedimento"), default)]
    pub customs_declaration_number: Option<String>,
    #[serde(rename(deserialize = "@IdentDocAduanero"), default)]
    pub customs_document_id: Option<String>,
    #[serde(rename(deserialize = "@RFCImpo"), default)]
    pub importer_tax_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingId {
    #[serde(rename(deserialize = "@NumeroGuiaIdentificacion"))]
    pub tracking_number: String,
    #[serde(rename(deserialize = "@DescripGuiaIdentificacion"))]
    pub description: String,
    #[serde(rename(deserialize = "@PesoGuiaIdentificacion"))]
    pub weight: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityTransport {
    #[serde(rename(deserialize = "@Cantidad"))]
    pub quantity: String,
    #[serde(rename(deserialize = "@IDOrigen"))]
    pub origin_id: String,
    #[serde(rename(deserialize = "@IDDestino"))]
    pub destination_id: String,
    #[serde(rename(deserialize = "@CvesTransporte"), default)]
    pub transport_keys: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchandiseDetail {
    #[serde(rename(deserialize = "@UnidadPesoMerc"))]
    pub weight_unit: String,
    #[serde(rename(deserialize = "@PesoBruto"))]
    pub gross_weight: String,
    #[serde(rename(deserialize = "@PesoNeto"))]
    pub net_weight: String,
    #[serde(rename(deserialize = "@PesoTara"))]
    pub tare_weight: String,
    #[serde(rename(deserialize = "@NumPiezas"), default)]
    pub pieces: Option<String>,
}

// ── Road transport (Autotransporte) — fully implemented ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadTransport {
    #[serde(rename(deserialize = "@PermSCT"))]
    pub sct_permit_type: String,
    #[serde(rename(deserialize = "@NumPermisoSCT"))]
    pub sct_permit_number: String,

    #[serde(rename(deserialize = "IdentificacionVehicular"))]
    pub vehicle_id: VehicleId,
    #[serde(rename(deserialize = "Seguros"))]
    pub insurance: VehicleInsurance,
    #[serde(rename(deserialize = "Remolques"), default)]
    pub trailers: Option<Trailers>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleId {
    #[serde(rename(deserialize = "@ConfigVehicular"))]
    pub configuration: String,
    #[serde(rename(deserialize = "@PesoBrutoVehicular"))]
    pub gross_weight: String,
    #[serde(rename(deserialize = "@PlacaVM"))]
    pub license_plate: String,
    #[serde(rename(deserialize = "@AnioModeloVM"))]
    pub model_year: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleInsurance {
    #[serde(rename(deserialize = "@AseguraRespCivil"))]
    pub civil_liability_insurer: String,
    #[serde(rename(deserialize = "@PolizaRespCivil"))]
    pub civil_liability_policy: String,
    #[serde(rename(deserialize = "@AseguraMedAmbiente"), default)]
    pub environmental_insurer: Option<String>,
    #[serde(rename(deserialize = "@PolizaMedAmbiente"), default)]
    pub environmental_policy: Option<String>,
    #[serde(rename(deserialize = "@AseguraCarga"), default)]
    pub cargo_insurer: Option<String>,
    #[serde(rename(deserialize = "@PolizaCarga"), default)]
    pub cargo_policy: Option<String>,
    #[serde(rename(deserialize = "@PrimaSeguro"), default)]
    pub insurance_premium: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trailers {
    #[serde(rename(deserialize = "Remolque"), default)]
    pub items: Vec<Trailer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trailer {
    #[serde(rename(deserialize = "@SubTipoRem"))]
    pub sub_type: String,
    #[serde(rename(deserialize = "@Placa"))]
    pub license_plate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederalRoadTransport {
    #[serde(rename(deserialize = "@PermSCT"))]
    pub sct_permit_type: String,
    #[serde(rename(deserialize = "@NumPermisoSCT"))]
    pub sct_permit_number: String,
}

// ── Shell structs for unimplemented modalities ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaritimeTransportShell {
    #[serde(rename(deserialize = "@PermSCT"), default)]
    pub sct_permit_type: Option<String>,
    #[serde(rename(deserialize = "@NumPermisoSCT"), default)]
    pub sct_permit_number: Option<String>,
    #[serde(rename(deserialize = "@NombreAseg"), default)]
    pub insurer_name: Option<String>,
    #[serde(rename(deserialize = "@NumPolizaSegur"), default)]
    pub insurance_policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirTransportShell {
    #[serde(rename(deserialize = "@PermSCT"), default)]
    pub sct_permit_type: Option<String>,
    #[serde(rename(deserialize = "@NumPermisoSCT"), default)]
    pub sct_permit_number: Option<String>,
    #[serde(rename(deserialize = "@MatriculaAeronave"), default)]
    pub aircraft_registration: Option<String>,
    #[serde(rename(deserialize = "@NombreAseg"), default)]
    pub insurer_name: Option<String>,
    #[serde(rename(deserialize = "@NumPolizaSegur"), default)]
    pub insurance_policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RailTransportShell {
    #[serde(rename(deserialize = "@TipoDeServicio"), default)]
    pub service_type: Option<String>,
    #[serde(rename(deserialize = "@TipoDeTrafico"), default)]
    pub traffic_type: Option<String>,
}

// ── Transport figures ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportFigures {
    #[serde(rename(deserialize = "TiposFigura"), default)]
    pub items: Vec<TransportFigure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportFigure {
    #[serde(rename(deserialize = "@TipoFigura"))]
    pub figure_type: String,
    #[serde(rename(deserialize = "@RFCFigura"), default)]
    pub taxpayer_id: Option<String>,
    #[serde(rename(deserialize = "@NumLicencia"), default)]
    pub license_number: Option<String>,
    #[serde(rename(deserialize = "@NombreFigura"), default)]
    pub name: Option<String>,
    #[serde(rename(deserialize = "@NumRegIdTribFigura"), default)]
    pub foreign_tax_id: Option<String>,
    #[serde(rename(deserialize = "@ResidenciaFiscalFigura"), default)]
    pub tax_residence: Option<String>,

    #[serde(rename(deserialize = "PartesTransporte"), default)]
    pub transport_parts: Vec<TransportPart>,
    #[serde(rename(deserialize = "Domicilio"), default)]
    pub address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportPart {
    #[serde(rename(deserialize = "@ParteTransporte"))]
    pub part: String,
}
