import axios from "axios";
import { Match } from "./types";
import { EventSource } from "eventsource"
import assert, { deepEqual } from "assert";

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
const URL = "http://localhost:8080"
class Client {
    token: string
    constructor() {
        this.token = generateRandomToken();
    }
    async createGame(): Promise<Match> {
        const prize = pickRandom(PRIZES)
        const game_type = pickRandom(GAME_TYPES)
        const response = await axios.post(`${URL}/create`,
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
        return new Promise<EventSource>((resolve) => {
            const es = new EventSource(`${URL}/updates?id=${id}`)
            es.onmessage = (event: any) => {
                console.log(`Client ${this.token} Received: ` + event.data);
            }
            es.onerror = (err: any) => {
                console.error(`Client ${this.token} Error:`, err);
            };
            es.onopen = (event: any) => {
                console.log("Event source opened: ", JSON.stringify(event));
                resolve(es);
            }
        })
    }
    async joinGame(id: number): Promise<Match> {
        const response = await axios.post(`${URL}/join?id=${id}`, null,
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
        const response = await axios.get(`${URL}/match?id=${id}`,
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
        const response = await axios.get(`${URL}/matches`,
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
        const response = await axios.post(`${URL}/ready?id=${id}`, null,
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