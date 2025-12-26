@0xdf98f828a1c9751e;

interface TaxEngine {
  calculate @0 (tx :TransactionRequest) -> (response :TaxResponse);
}

struct TransactionRequest {
  clientId @0 :Text;
  amount @1 :Float64;
  jurisdiction @2 :Text;
  product @3 :Text;
  # Date could be added here as text or int64 (timestamp)
}

struct TaxResponse {
  totalAmount @0 :Float64;
  breakdown @1 :List(TaxDetail);
}

struct TaxDetail {
  taxType @0 :Text;
  base @1 :Float64;
  rate @2 :Float64;
  amount @3 :Float64;
}
