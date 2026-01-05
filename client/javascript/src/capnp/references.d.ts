import * as $ from "capnp-es";
export declare const _capnpFileId: bigint;
export declare class TaxEngine_Calculate$Params extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  _adoptTx(value: $.Orphan<TransactionRequest>): void;
  _disownTx(): $.Orphan<TransactionRequest>;
  get tx(): TransactionRequest;
  _hasTx(): boolean;
  _initTx(): TransactionRequest;
  set tx(value: TransactionRequest);
  toString(): string;
}
export declare class TaxEngine_Calculate$Results extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  _adoptResponse(value: $.Orphan<TaxResponse>): void;
  _disownResponse(): $.Orphan<TaxResponse>;
  get response(): TaxResponse;
  _hasResponse(): boolean;
  _initResponse(): TaxResponse;
  set response(value: TaxResponse);
  toString(): string;
}
export declare class TaxEngine_Calculate$Results$Promise {
  pipeline: $.Pipeline<any, any, TaxEngine_Calculate$Results>;
  constructor(pipeline: $.Pipeline<any, any, TaxEngine_Calculate$Results>);
  promise(): Promise<TaxEngine_Calculate$Results>;
}
export declare class TaxEngine$Client {
  client: $.Client;
  static readonly interfaceId: bigint;
  constructor(client: $.Client);
  static readonly methods: [
    $.Method<TaxEngine_Calculate$Params, TaxEngine_Calculate$Results>
  ];
  calculate(paramsFunc?: (params: TaxEngine_Calculate$Params) => void): TaxEngine_Calculate$Results$Promise;
}
export interface TaxEngine$Server$Target {
  calculate(params: TaxEngine_Calculate$Params, results: TaxEngine_Calculate$Results): Promise<void>;
}
export declare class TaxEngine$Server extends $.Server {
  readonly target: TaxEngine$Server$Target;
  constructor(target: TaxEngine$Server$Target);
  client(): TaxEngine$Client;
}
export declare class TaxEngine extends $.Interface {
  static readonly Client: typeof TaxEngine$Client;
  static readonly Server: typeof TaxEngine$Server;
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  toString(): string;
}
export declare class TransactionRequest extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get clientId(): string;
  set clientId(value: string);
  get amount(): number;
  set amount(value: number);
  get jurisdiction(): string;
  set jurisdiction(value: string);
  get product(): string;
  set product(value: string);
  toString(): string;
}
export declare class TaxResponse extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  static _Breakdown: $.ListCtor<TaxDetail>;
  get totalAmount(): number;
  set totalAmount(value: number);
  _adoptBreakdown(value: $.Orphan<$.List<TaxDetail>>): void;
  _disownBreakdown(): $.Orphan<$.List<TaxDetail>>;
  get breakdown(): $.List<TaxDetail>;
  _hasBreakdown(): boolean;
  _initBreakdown(length: number): $.List<TaxDetail>;
  set breakdown(value: $.List<TaxDetail>);
  toString(): string;
}
export declare class TaxDetail extends $.Struct {
  static readonly _capnp: {
    displayName: string;
    id: string;
    size: $.ObjectSize;
  };
  get taxType(): string;
  set taxType(value: string);
  get base(): number;
  set base(value: number);
  get rate(): number;
  set rate(value: number);
  get amount(): number;
  set amount(value: number);
  toString(): string;
}
