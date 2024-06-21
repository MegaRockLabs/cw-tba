import type { SigningArchwayClient } from "@archwayhq/arch3.js";

export const createAccountMsg = () => {
    const execute_msg  = {
        create_account: {
            chain_id: process.env.PUBLIC_CHAIN_ID!,
            code_id: process.env.ACCOUNT_ID!,
            msg: {
                auth_data: {
                    credentials: [],
                },
            }
        }
    }

    return execute_msg;
}

export const getCWFee = async (
    client: SigningArchwayClient, 
    signer: string
) : Promise<any> => {
    return "auto"
}