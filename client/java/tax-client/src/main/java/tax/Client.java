package tax;

import tax_engine.TaxEngineClientFactory;

import org.capnproto.TwoPartyClient;
import org.capnproto.RpcException;

import java.net.InetSocketAddress;
import java.nio.channels.AsynchronousSocketChannel;
import java.util.concurrent.ExecutionException;

public class Client {
    public static void main(String[] args) throws Exception {

        String host = "127.0.0.1";
        int port = 50051;
        String clientId = "client_1";
        double amount = 1000.0;
        String jurisdiction = "TEST_J";
        String product = "TEST_PROD";

        try (var clientSocket = AsynchronousSocketChannel.open()) {
            clientSocket.connect(new InetSocketAddress(host, port)).get();
            var rpcClient = new TwoPartyClient(clientSocket);
            var engine = new TaxEngineClientFactory.TaxEngine.Client(rpcClient.bootstrap());

            // Build calculate request
            var request = engine.calculateRequest();
            var params = request.getParams();
            var tx = params.initTx();
            tx.setClientId(clientId);
            tx.setAmount(amount);
            tx.setJurisdiction(jurisdiction);
            tx.setProduct(product);

            System.out.println("Built request for clientId: " + tx.getClientId());

            try {
                // Send the request and run the RPC system until we get the results
                var responseFuture = request.send();
                var resultsFuture = rpcClient.runUntil(responseFuture);
                var response = resultsFuture.join();

                if (response != null && response.hasResponse()) {
                    var res = response.getResponse();
                    System.out.println("Total amount: " + res.getTotalAmount());
                    if (res.hasBreakdown()) {
                        var breakdown = res.getBreakdown();
                        for (int i = 0; i < breakdown.size(); i++) {
                            var data = breakdown.get(i);
                            System.out.printf("Tax: %s | Rate: %s | Amount: %s%n",
                                    data.getTaxType().toString(), data.getRate(), data.getAmount());
                        }
                    }
                } else {
                    System.err.println("No response or empty results from tax engine");
                }
            } catch (java.util.concurrent.CompletionException ce) {
                var cause = ce.getCause();
                if (cause instanceof RpcException) {
                    System.err.println("RPC error from server: " + cause.getMessage());
                } else {
                    System.err.println("Execution failed: " + ce.getMessage());
                    ce.printStackTrace();
                }
            }

        } catch (InterruptedException | ExecutionException e) {
            System.err.println("RPC connection/setup failed: " + e.getMessage());
            e.printStackTrace();
        }
    }
}