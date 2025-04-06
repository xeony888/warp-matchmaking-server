import axios from "axios";
import { Match } from "./types";
import { EventSource } from "eventsource"
import assert, { deepEqual } from "assert";
import dotenv from "dotenv";
import { WebSocket } from "ws";
dotenv.config();
axios.defaults.validateStatus = (status) => {
    return true;
}
const letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabdefghijklmnopqrstuvwxyz"
function generateRandomToken(): string {
    return Array.from({ length: 20 }).map((_) => letters[Math.floor(Math.random() * letters.length)]).join("");
}
const PRIZES: number[] = [2, 5, 10, 25, 50]
const GAME_TYPES: string[] = ["soccer", "knockout"]
function pickRandom<T>(arr: T[]): T {
    return arr[Math.floor(Math.random() * arr.length)];
}
class Client {
    token: string
    url: string;
    constructor() {
        this.token = generateRandomToken();
        this.url = `http://${process.env.HOST}:${process.env.PORT}`
        console.log(this.url);
    }
    async createGame(): Promise<Match> {
        const prize = pickRandom(PRIZES)
        const game_type = pickRandom(GAME_TYPES)
        const response = await axios.post(`${this.url}/create`,
            { prize, game_type },
            {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${this.token}`,
                },
            }
        )
        assert(response.status === 200, "Invalid response status");
        const { id, players, prize: gamePrize, game_type: gameGame_type, expiry_time } = response.data as Match;
        assert(players.includes(this.token), "Players does not include token");
        assert(Number(gamePrize) === prize, "Incorrect prize value");
        assert(gameGame_type === game_type, "Invalid game type");
        return response.data as Match;
    }
    async openEventSource(id: number): Promise<EventSource> {
        return new Promise<EventSource>((resolve, reject) => {
            const es = new EventSource(`${this.url}/updates?id=${id}`)
            es.onmessage = (event: any) => {
                const eventData = JSON.parse(event.data);
                console.log(eventData);
                const state = eventData[0]
                const port = eventData[3];
                if (state === "PLAYING") {
                    const wsUrl = `http://localhost:${port}`;
                    console.log(`Connecting to game on ${wsUrl}`);
                    const websocket = new WebSocket(wsUrl);
                    websocket.onopen = (event) => {
                        console.log("Websocket opened with url: " + wsUrl);
                        console.log("WS Data: " + String(event));
                    };
                    websocket.onerror = (event) => {
                        console.log(`Websocket errored with error: ${event.error}, message: ${event.message}, target: ${JSON.stringify(event.target)}, type: ${event.type}`);
                    }
                }
                console.log(`Client ${this.token} Received: ` + event.data);
            }
            es.onerror = (err: any) => {
                console.error(`Client ${this.token} Error:`, err);
                reject();
            };
            es.onopen = (event: any) => {
                console.log("Event source opened: ", JSON.stringify(event));
                resolve(es);
            }
        })
    }
    async joinGame(id: number): Promise<Match> {
        const response = await axios.post(`${this.url}/join?id=${id}`, null,
            {
                method: "POST",
                headers: {
                    Authorization: `Bearer ${this.token}`
                },
            }
        )
        assert(response.status === 200, "Invalid response status");
        const { id: game_id, game_type, prize, expiry_time, players } = response.data as Match;
        assert(game_id === id, "Invalid game id");
        return response.data as Match
    }
    async getGame(id: number): Promise<Match> {
        const response = await axios.get(`${this.url}/match?id=${id}`,
            {
                method: "GET",
                headers: {
                    Authorization: `Bearer ${this.token}`,
                }
            }
        )
        assert(response.status === 200, "Invalid response status");
        return response.data
    }
    async getGames(): Promise<Match[]> {
        const response = await axios.get(`${this.url}/matches`,
            {
                method: "GET",
                headers: {
                    Authorization: `Bearer ${this.token}`,
                }
            }
        )
        assert(response.status === 200, "Invalid response status");
        return response.data;
    }
    async readyUp(id: number): Promise<boolean> {
        const response = await axios.post(`${this.url}/ready?id=${id}`, null,
            {
                method: "POST",
                headers: {
                    Authorization: `Bearer ${this.token}`
                }
            }
        )
        assert(response.status === 200, "Invalid response status");
        return true;
    }
}
async function main() {
    const client1 = new Client();
    const client2 = new Client();

    const match = await client1.createGame();
    const es = await client1.openEventSource(match.id);
    await client2.joinGame(match.id);
    const es2 = await client2.openEventSource(match.id);
    await Promise.all([client1.readyUp(match.id), client2.readyUp(match.id)]);
}
main().then(() => console.log("DONE"));