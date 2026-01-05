import { connectTaxEngine } from "./connection.js";
import type { TaxEngine_Calculate$Params as Params } from "./capnp/references.js";

async function main() {

  try {
    const connection = await connectTaxEngine("127.0.0.1:50051");

    const execution = await connection.client
      .calculate((params: Params) => {
        const tx = params._initTx();
        tx.clientId = "client_1";
        tx.amount = 100.0;
        tx.jurisdiction = "test_j";
        tx.product = "TEST_PROD";
      }).promise();

    const response = execution.response;
    console.log("Total: ", response.totalAmount);

    const breakdown = response.breakdown || [];
    for (let i = 0; i < breakdown.length; i++) {

      const d = breakdown.get(i);

      console.log("Tax:", {
        taxType: d.taxType,
        base: d.base,
        rate: d.rate,
        amount: d.amount,
      });
    }

    connection.close();

  } catch (error) {
    console.log(error);
  }
}

main();