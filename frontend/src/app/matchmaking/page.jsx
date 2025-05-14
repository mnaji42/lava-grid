"use client"

import { useEffect, useRef, useState } from "react"
import { useRouter } from "next/navigation"

export default function MatchmakingPage({ wallet }) {
  const [ws, setWs] = useState(null)
  const [players, setPlayers] = useState([])
  const [countdown, setCountdown] = useState(null)
  const [username, setUsername] = useState("")
  const router = useRouter()
  const countdownRef = useRef(null)

  // Charger le pseudo depuis le localStorage
  useEffect(() => {
    if (typeof window !== "undefined") {
      const savedUsername = localStorage.getItem("username") || ""
      setUsername(savedUsername)
    }
  }, [])

  // Sauvegarder le pseudo
  useEffect(() => {
    if (username) localStorage.setItem("username", username)
  }, [username])

  // Si pas de wallet, ne rien afficher (la page parent gère le login)
  if (!wallet) return null

  const connect = () => {
    const socket = new WebSocket(
      `ws://localhost:8080/ws/matchmaking?wallet=${encodeURIComponent(
        wallet
      )}&username=${encodeURIComponent(username)}`
    )

    socket.onopen = () => console.log("Connected to matchmaking")

    socket.onmessage = (event) => {
      const msg = JSON.parse(event.data)
      switch (msg.action) {
        case "PlayerJoin":
        case "PlayerLeave":
        case "UpdateState":
          const state = msg.data
          setPlayers(
            state.players.map((p) => ({
              username: p.username,
              wallet: p.id,
            }))
          )

          if (state.countdown_active) {
            startCountdown(state.time_remaining)
          } else {
            clearInterval(countdownRef.current)
            setCountdown(null)
          }
          break

        case "GameStarted":
          ws?.close()
          router.push(`/game/${msg.data.game_id}`)
          break

        default:
          console.warn("Unhandled message type:", msg.action)
      }
    }

    socket.onclose = () => {
      console.log("Disconnected from matchmaking")
      clearInterval(countdownRef.current)
      setCountdown(null)
      setPlayers([])
    }

    setWs(socket)
  }

  const disconnect = () => {
    ws?.close()
    setWs(null)
  }

  const startCountdown = (seconds) => {
    setCountdown(seconds)
    countdownRef.current = setInterval(() => {
      setCountdown((prev) => {
        if (prev <= 1) {
          clearInterval(countdownRef.current)
          return null
        }
        return prev - 1
      })
    }, 1000)
  }

  return (
    <div className="min-h-screen bg-gradient-to-b from-slate-900 to-slate-800 flex items-center justify-center p-4">
      {!ws ? (
        <div className="bg-white rounded-2xl p-8 shadow-xl w-full max-w-md space-y-6 animate-fade-in">
          <h1 className="text-3xl font-bold text-center text-slate-800">
            Ready to Play?
          </h1>

          <div className="space-y-4">
            <input
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="Enter your username"
              className="w-full px-4 py-3 rounded-lg border-2 border-slate-200 focus:border-blue-500 focus:ring-2 focus:ring-blue-200 transition-all outline-none"
            />

            <button
              onClick={connect}
              disabled={!username.trim()}
              className="w-full py-3 px-6 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed text-white font-semibold rounded-lg transition-all transform hover:scale-[1.02] active:scale-95"
            >
              Play Now
            </button>
          </div>
        </div>
      ) : (
        <div className="bg-white rounded-2xl p-8 shadow-xl w-full max-w-md space-y-6 animate-fade-in">
          <div className="flex justify-between items-center">
            <h2 className="text-2xl font-bold text-slate-800">
              Matchmaking Lobby
            </h2>
            <span className="bg-green-100 text-green-800 px-3 py-1 rounded-full text-sm">
              {players.length} Players
            </span>
          </div>

          <div className="mb-2">
            <span className="text-xs text-slate-500">Your wallet:</span>
            <span className="font-mono bg-slate-100 px-2 py-1 rounded text-slate-800 text-xs ml-2">
              {wallet}
            </span>
          </div>

          {countdown !== null && (
            <div className="text-center space-y-2">
              <p className="text-sm text-slate-500">Game starting in</p>
              <div className="text-4xl font-bold text-blue-600 animate-pulse">
                {countdown}s
              </div>
            </div>
          )}

          <div className="space-y-4">
            <div className="bg-slate-50 p-4 rounded-lg">
              <h3 className="font-medium text-slate-600 mb-2">Players</h3>
              <div className="space-y-2">
                {players.map((player) => (
                  <div
                    key={player.wallet}
                    className={`flex items-center p-2 rounded-md ${
                      player.wallet === wallet
                        ? "bg-blue-100 border border-blue-200"
                        : "bg-white"
                    }`}
                  >
                    <div className="h-2 w-2 rounded-full bg-green-400 mr-3" />
                    <span className="font-medium">{player.username}</span>
                    <span className="ml-2 font-mono text-xs text-slate-400">
                      {player.wallet.slice(0, 8)}...
                    </span>
                    {player.wallet === wallet && (
                      <span className="ml-2 px-2 py-1 bg-blue-600 text-white text-xs rounded-full">
                        You
                      </span>
                    )}
                  </div>
                ))}
              </div>
            </div>

            <button
              onClick={disconnect}
              className="w-full py-2 px-6 bg-red-600 hover:bg-red-700 text-white font-semibold rounded-lg transition-all transform hover:scale-[1.02] active:scale-95"
            >
              Leave Matchmaking
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
