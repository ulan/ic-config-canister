import {
    blob,
    Canister,
    init,
    nat16,
    Opt,
    query,
    Record,
    StableBTreeMap,
    text,
    Tuple,
    update,
    Vec
} from 'azle';


type HeaderField = [text, text];
const HeaderField = Tuple(text, text);

const HttpResponse = Record({
    status_code: nat16,
    headers: Vec(HeaderField),
    body: blob,
});

const HttpRequest = Record({
    method: text,
    url: text,
    headers: Vec(HeaderField),
    body: blob,
    certificate_version: Opt(nat16)
});

let configByReplicaVersion = StableBTreeMap<text, text>(text, text, 0);

export default Canister({
    init: init([], () => { }),
    http_request: query([HttpRequest], HttpResponse, (req) => {
        if (req.method != 'GET') {
            return error('Only GET requests are supported');
        }

        if (req.url.startsWith('/versions')) {
            const keys = configByReplicaVersion.keys();
            return {
                status_code: 200,
                headers: [['content-type', 'application/json']],
                body: encode(JSON.stringify(keys)),
            };
        }

        if (req.url.startsWith('/config')) {
            const match = /version=[" ]?([^&" ]+)/g.exec(req.url);
            if (!match || !match[1]) {
                return error('Expected a version argument: ?version=<replica_version>');
            }
            const version = match[1];
            const maybeConfig = configByReplicaVersion.get(version);
            if (!maybeConfig.Some) {
                return error(`No config for the given version: ${version}`);
            }
            const config = maybeConfig.Some;
            return {
                status_code: 200,
                headers: [['content-type', 'application/json']],
                body: encode(config),
            };
        }

        return {
            status_code: 404,
            headers: [['content-type', 'application/json']],
            body: encode('Supported endpoints: /versions and /config'),
        };

    }),

    add: update([text, text], text, (version, config) => {
        version = version.trim();
        try {
            const json = JSON.parse(config);
            config = JSON.stringify(json);
        } catch (err) {
            return `The given config is not a valid JSON: ${err}`;
        }
        configByReplicaVersion.insert(version, config);
        return `Added config for version ${version}`;
    }),
});


function encode(string: string): blob {
    return Buffer.from(string, 'utf-8');
}

function error(msg: text): typeof HttpResponse {
    return {
        status_code: 400,
        headers: [],
        body: encode(msg),
    };
}
