<script lang="ts">
    import {
        CredentialType,
        Instructions,
        Workflow,
        Claim,
        retrieveSigner,
        signerMap,
        claims,
        getSubject,
        witnessState,
        sign,
        client,
        Signer,
        lookUp,
        alert,
        needsDelimitor,
        signerMapAppend,
        toQuery,
    } from "src/util";
    import Ajv from "ajv";
    import { onDestroy, onMount } from "svelte";
    import { useNavigate } from "svelte-navigator";
    import {
        WitnessFormHeader,
        Button,
        // NewSignerSelector,
        SignerConnect,
    } from "src/components";
    import WitnessFormStatement from "./WitnessFormStatement.svelte";
    import WitnessFormSignature from "./WitnessFormSignature.svelte";
    import WitnessFormWitness from "./WitnessFormWitness.svelte";
    import WitnessFormComplete from "./WitnessFormComplete.svelte";

    let _lookUp = null;
    lookUp.subscribe((x) => (_lookUp = x));
    let _signerMap = null;
    signerMap.subscribe((x) => (_signerMap = x));

    const navigate = useNavigate();
    const ajv = new Ajv();

    let statement_schema = null,
        witness_schema = null;
    const dnsPrefix = "rebase_sig=";

    let verified: boolean = false;
    let loading: boolean = false;

    let c: Array<Claim> = [];
    claims.subscribe((x) => (c = x));

    const handleNewSigner = (signer: Signer) => {
        signerMap.set(signerMapAppend(signer, _signerMap));
        lookUp.set(toQuery(signer));
        advance();
    };

    const setNew = (credential: string) => {
        let next: Array<Claim> = [];
        for (let i = 0, n = c.length; i < n; i++) {
            let claim = c[i];
            if (claim.credential_type === type) {
                claim.credentials.push(credential);
            }
            next.push(claim);
        }
        claims.set(next);
    };

    export let type: CredentialType;
    export let instructions: Instructions;

    $: statement = "";
    $: signature = "";
    $: delimitor = "";
    $: handle = "";
    $: proof = "";

    const onChangeValue = (name, value) => {
        switch (name) {
            case "handle":
                handle = value;
                break;
            case "signature":
                signature = value;
                break;
            case "proof":
                proof = value;
                break;
            case "verified":
                verified = value;
                break;
        }
    };

    const post = (): string => {
        switch (type) {
            case "discord":
            case "github":
            case "twitter":
                return `${statement}${delimitor}${signature}`;
            case "dns":
                return `${dnsPrefix}${signature}`;
            case "soundcloud":
            case "reddit":
                return `${signature}`;
            case "email":
                return;
        }
    };

    let state: Workflow = "signer";
    witnessState.subscribe((x) => (state = x));

    onMount(async () => {
        if (_lookUp) {
            witnessState.set("statement");
        } else {
            witnessState.set("signer");
        }
        try {
            let res = await client.instructions(JSON.stringify({ type }));
            let instruction_res = JSON.parse(res);
            statement_schema = instruction_res?.statement_schema;
            witness_schema = instruction_res?.witness_schema;
        } catch (err) {
            alert.set({
                message: "Failed to retrieve instructions from witness service",
                variant: "error",
            });
        }
    });

    onDestroy(() => {
        witnessState.set("signer");
    });

    const advance = () => {
        if (!_lookUp) {
            return witnessState.set("signer");
        }
        switch (state) {
            case "signer":
                return witnessState.set("statement");
            case "statement":
                return witnessState.set("signature");
            case "signature":
                return witnessState.set("witness");
            case "witness":
                return witnessState.set("complete");
            case "complete":
                return;
        }
    };

    const back = () => {
        switch (state) {
            case "statement":
                return witnessState.set("signer");
            case "signature":
                return witnessState.set("statement");
            case "witness":
                return witnessState.set("signature");
            case "complete":
                return witnessState.set("witness");
        }
    };

    const getStatement = async (): Promise<void> => {
        let opts = {};
        opts[type] = {};
        let current = retrieveSigner(_signerMap, _lookUp);
        if (!current) {
            throw new Error("No default signer set");
        }

        switch (type) {
            case "dns":
                opts[type]["domain"] = handle;
                opts[type]["prefix"] = dnsPrefix;
                opts[type]["subject"] = getSubject(current);
                break;
            case "github":
            case "twitter":
            case "reddit":
                opts[type]["handle"] = handle;
                opts[type]["subject"] = getSubject(current);
                break;
            case "soundcloud":
                opts[type]["permalink"] =
                    handle.split("/")[handle.split("/").length - 1];
                opts[type]["subject"] = getSubject(current);
                break;
            case "email":
                opts[type]["email"] = handle;
                opts[type]["subject"] = getSubject(current);
                break;
            default:
                throw new Error(`${type} flow is currently unsupported`);
        }

        if (!statement_schema) {
            throw new Error("No JSON Schema found for Statement Request");
        }

        if (!ajv.validate(statement_schema, opts[type])) {
            throw new Error("Validation of Statement Request failed");
        }

        const badRespErr = "Badly formatted witness service response";
        try {
            let res = await client.statement(JSON.stringify({ opts }));

            let body = JSON.parse(res);
            if (!body.statement) {
                throw new Error(badRespErr + " missing statement");
            }

            if (needsDelimitor(type) && !body.delimitor) {
                throw new Error(badRespErr + " missing delimitor");
            }

            statement = body.statement;
            delimitor = body.delimitor;
        } catch (e) {
            if (e.message === badRespErr) {
                throw e;
            } else {
                throw new Error("Failed in request for statement to witness");
            }
        }
    };

    const getCredential = async (): Promise<void> => {
        let opts = {};

        let current = retrieveSigner(_signerMap, _lookUp);
        if (!current) {
            throw new Error("No default signer set");
        }

        switch (type) {
            case "dns":
                opts["dns"] = {};
                opts["dns"]["domain"] = handle;
                opts["dns"]["prefix"] = dnsPrefix;
                opts["dns"]["subject"] = getSubject(current);
                break;
            case "reddit":
                opts["reddit"] = {};
                opts["reddit"]["handle"] = handle;
                opts["reddit"]["subject"] = getSubject(current);
                break;
            case "soundcloud":
                opts["soundcloud"] = {};
                opts["soundcloud"]["permalink"] =
                    handle.split("/")[handle.split("/").length - 1];
                opts["soundcloud"]["subject"] = getSubject(current);
                break;
            case "github":
                opts["github"] = {};
                opts["github"]["statement"] = {};
                opts["github"]["statement"]["handle"] = handle;
                opts["github"]["statement"]["subject"] = getSubject(current);
                opts["github"]["gist_id"] = proof.split("/").pop();
                break;
            case "twitter":
                opts["twitter"] = {};
                opts["twitter"]["statement"] = {};
                opts["twitter"]["statement"]["handle"] = handle;
                opts["twitter"]["statement"]["subject"] = getSubject(current);
                opts["twitter"]["tweet_url"] = proof.split("?")[0];
                break;
            case "email":
                opts["email"] = {};
                opts["email"]["statement"] = {};
                opts["email"]["statement"]["email"] = handle;
                opts["email"]["statement"]["subject"] = getSubject(current);
                opts["email"]["auth"] = proof.split(":::")[0].trim();
                opts["email"]["timestamp"] = proof.split(":::")[1].trim();
                opts["email"]["signature"] = signature;
                break;
            default:
                throw new Error(`${type} flow is currently unsupported`);
        }

        if (!witness_schema) {
            throw new Error("No JSON Schema found for Witness Request");
        }

        if (!ajv.validate(witness_schema, opts[type])) {
            throw new Error("Validation of Witness Request failed");
        }

        try {
            let b = JSON.stringify({ proof: opts });
            let res = await client.jwt(b);

            let { jwt } = JSON.parse(res);
            setNew(jwt);
        } catch (e) {
            throw new Error(
                "Failed to issue credential, please retry the flow"
            );
        }
    };
</script>

<WitnessFormHeader
    icon={instructions.icon}
    title={instructions.title}
    subtitle={instructions.subtitle}
/>
{#if state === "signer"}
    <div class="w-full">
        <div class="flex px-4 text-center">
            <div class="w-full">
                <h4>Please connect a signer to link</h4>
                <SignerConnect
                    primary
                    class="menu w-full min-w-42 mt-[8px] rounded-xl"
                    signerCallback={handleNewSigner}
                >
                    Connect signer
                </SignerConnect>
            </div>
        </div>
    </div>
    <div
        class="w-full my-[16px] text-center  flex flex-wrap justify-evenly items-center content-end"
    >
        <Button
            class="w-2/5"
            disabled={!_lookUp}
            onClick={advance}
            text="Next"
            action
        />
    </div>
{/if}
{#if state === "statement"}
    <WitnessFormStatement
        {instructions}
        {loading}
        {handle}
        {onChangeValue}
        {navigate}
        {getStatement}
        {advance}
    />
{/if}
{#if state === "signature"}
    <WitnessFormSignature
        {instructions}
        {loading}
        {statement}
        {signature}
        {onChangeValue}
        {sign}
        {back}
        {advance}
    />
{/if}
{#if state === "witness"}
    <WitnessFormWitness
        {instructions}
        {loading}
        {verified}
        {type}
        {proof}
        {onChangeValue}
        {getCredential}
        {post}
        {back}
        {advance}
    />
{/if}
{#if state === "complete"}
    <WitnessFormComplete {navigate} />
{/if}
