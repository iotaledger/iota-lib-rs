package node_api_core;

import org.iota.Client;
import org.iota.types.ClientConfig;
import org.iota.types.ClientException;
import org.iota.types.secret.GenerateAddressesOptions;
import org.iota.types.secret.MnemonicSecretManager;

public class GenerateAddresses {
    public static void main(String[] args) throws ClientException {
        // Build the client.
        Client client = new Client(new ClientConfig("{ \"nodes\": [ \"https://api.testnet.shimmer.network\" ], \"nodeSyncEnabled\": true }"));

        // Generate the addresses.
        MnemonicSecretManager secretManager = new MnemonicSecretManager("endorse answer radar about source reunion marriage tag sausage weekend frost daring base attack because joke dream slender leisure group reason prepare broken river");
        String[] addresses = client.generateAddresses(secretManager, new GenerateAddressesOptions().withRange(0, 5));

        // Print the addresses.
        for (String address : addresses) {
            System.out.println(address);
        }

    }
}