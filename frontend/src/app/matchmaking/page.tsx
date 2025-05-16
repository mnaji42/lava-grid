"use client"

import { useEffect, useRef, useState } from "react"
import { useRouter } from "next/navigation"

type Player = {
  id: string
  username: string
}

type MatchmakingState = {
  lobby_players: Player[]
  ready_players: Player[]
  countdown_active: boolean
  countdown_remaining: number | null
}

function ActionButtons({
  isPaid,
  isConnected,
  countdown,
  onPay,
  onCancelPayment,
  onLeave,
}: {
  isPaid: boolean
  isConnected: boolean
  countdown: number | null
  onPay: () => void
  onCancelPayment: () => void
  onLeave: () => void
}) {
  const leaveDisabled = isPaid && !!countdown
  const showPay = !isPaid && isConnected
  const showCancel = isPaid
  const cancelDisabled = !!countdown

  const getButtonClass = (base: string, disabled: boolean) =>
    `${base} ${disabled ? "opacity-50 cursor-not-allowed" : ""}`

  return (
    <div className="flex flex-col gap-2">
      {showPay && (
        <button
          onClick={onPay}
          className="w-full py-2 px-6 bg-green-600 hover:bg-green-700 text-white font-semibold rounded-lg transition-all"
        >
          Pay to join the game
        </button>
      )}
      {showCancel && (
        <button
          onClick={onCancelPayment}
          className={getButtonClass(
            "w-full py-2 px-6 bg-yellow-500 hover:bg-yellow-600 text-white font-semibold rounded-lg transition-all",
            cancelDisabled
          )}
          disabled={cancelDisabled}
        >
          Cancel payment
        </button>
      )}
      <button
        onClick={onLeave}
        className={getButtonClass(
          "w-full py-2 px-6 bg-red-600 hover:bg-red-700 text-white font-semibold rounded-lg transition-all",
          leaveDisabled
        )}
        disabled={leaveDisabled}
      >
        Leave matchmaking
      </button>
    </div>
  )
}

function CryptoPaymentModal({
  open,
  onConfirm,
  onCancel,
}: {
  open: boolean
  onConfirm: () => void
  onCancel: () => void
}) {
  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-40">
      <div className="bg-white rounded-xl shadow-lg p-8 max-w-sm w-full animate-fade-in">
        <h2 className="text-xl font-bold mb-4 text-center text-slate-800">
          Signature de la transaction
        </h2>
        <p className="mb-6 text-slate-600 text-center">
          Pour rejoindre la partie, veuillez signer la transaction crypto dans
          votre wallet.
        </p>
        <div className="flex gap-4 justify-center">
          <button
            onClick={onCancel}
            className="px-4 py-2 rounded bg-slate-200 hover:bg-slate-300 text-slate-700 font-semibold transition"
          >
            Annuler
          </button>
          <button
            onClick={onConfirm}
            className="px-4 py-2 rounded bg-blue-600 hover:bg-blue-700 text-white font-semibold transition"
          >
            Signer et payer
          </button>
        </div>
      </div>
    </div>
  )
}

export default function MatchmakingPage({ wallet }) {
  const [ws, setWs] = useState<WebSocket | null>(null)
  const [connectedPlayers, setConnectedPlayers] = useState<Player[]>([])
  const [paidPlayers, setPaidPlayers] = useState<Player[]>([])
  const [countdown, setCountdown] = useState<number | null>(null)
  const [username, setUsername] = useState("")
  const [isPaid, setIsPaid] = useState(false)
  const [isConnected, setIsConnected] = useState(false)
  const [showPaymentModal, setShowPaymentModal] = useState(false)
  const router = useRouter()
  const countdownEndRef = useRef<number | null>(null)
  const timerRef = useRef<any>(null)

  // Load username from localStorage
  useEffect(() => {
    if (typeof window !== "undefined") {
      const savedUsername = localStorage.getItem("username") || ""
      setUsername(savedUsername)
    }
  }, [])

  // Save username to localStorage
  useEffect(() => {
    if (username) localStorage.setItem("username", username)
  }, [username])

  // If no wallet, render nothing (parent page handles login)
  if (!wallet) return null

  const connect = () => {
    const socket = new WebSocket(
      `ws://localhost:8080/ws/matchmaking?wallet=${encodeURIComponent(
        wallet
      )}&username=${encodeURIComponent(username)}`
    )

    socket.onopen = () => {
      setIsConnected(true)
    }

    socket.onmessage = (event) => {
      const msg = JSON.parse(event.data)
      console.log(msg)
      switch (msg.action) {
        case "UpdateState": {
          const state: MatchmakingState = msg.data
          setConnectedPlayers(state.lobby_players)
          setPaidPlayers(state.ready_players)
          setIsPaid(state.ready_players.some((p) => p.id === wallet))
          setIsConnected(state.lobby_players.some((p) => p.id === wallet))
          if (state.countdown_active && state.countdown_remaining !== null) {
            startCountdown(state.countdown_remaining)
          } else {
            clearTimer()
            setCountdown(null)
          }
          break
        }
        case "GameStarted":
          ws?.close()
          router.push(`/game/${msg.data.game_id}`)
          break
        case "Error":
          alert(msg.data?.message || "An error occurred")
          break
        default:
          break
      }
    }

    socket.onclose = () => {
      clearTimer()
      setCountdown(null)
      setConnectedPlayers([])
      setPaidPlayers([])
      setIsPaid(false)
      setIsConnected(false)
      setWs(null)
    }

    setWs(socket)
  }

  const disconnect = () => {
    ws?.close()
    setWs(null)
  }

  // Send a WS message to the server
  const sendWs = (msg: any) => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(msg))
    }
  }

  // Pay to join the paid lobby
  const pay = () => sendWs({ action: "Pay" })
  // Cancel payment (return to normal lobby)
  const cancelPayment = () => sendWs({ action: "CancelPayment" })

  // Accurate timer based on Date.now()
  const startCountdown = (seconds: number) => {
    clearTimer()
    const end = Date.now() + seconds * 1000
    countdownEndRef.current = end
    setCountdown(seconds)
    const tick = () => {
      const now = Date.now()
      const remaining = Math.max(
        0,
        Math.round(((countdownEndRef.current ?? 0) - now) / 1000)
      )
      setCountdown(remaining)
      if (remaining > 0) {
        timerRef.current = setTimeout(tick, 250)
      } else {
        clearTimer()
      }
    }
    timerRef.current = setTimeout(tick, 250)
  }

  const clearTimer = () => {
    if (timerRef.current) clearTimeout(timerRef.current)
    timerRef.current = null
    countdownEndRef.current = null
  }

  const handlePayClick = () => {
    setShowPaymentModal(true)
  }

  const handleConfirmPayment = () => {
    setShowPaymentModal(false)
    pay()
  }

  const handleCancelPayment = () => {
    setShowPaymentModal(false)
  }

  useEffect(() => {
    return () => clearTimer()
  }, [])

  // UX: dynamic wording based on player state
  const getLobbyStatus = () => {
    if (isPaid) {
      if (countdown !== null) {
        return (
          <>
            <p className="text-blue-700 font-semibold">
              Payment confirmed! The game will start soon.
            </p>
            <p className="text-slate-500 text-sm">
              You can no longer cancel your participation.
            </p>
          </>
        )
      }
      return (
        <>
          <p className="text-green-700 font-semibold">
            You are ready to play! Waiting for other paid players...
          </p>
          <p className="text-slate-500 text-sm">
            You can still cancel your payment as long as the countdown hasn't
            started.
          </p>
        </>
      )
    }
    if (isConnected) {
      return (
        <>
          <p className="text-slate-700 font-semibold">
            Join the game by confirming your payment.
          </p>
          <p className="text-slate-500 text-sm">
            Only players who have paid will be selected for the next game.
          </p>
        </>
      )
    }
    return (
      <p className="text-slate-500">
        Enter a username and connect to join matchmaking.
      </p>
    )
  }

  // Main render
  return (
    <div className="min-h-screen bg-gradient-to-b from-slate-900 to-slate-800 flex items-center justify-center p-4">
      {!ws ? (
        <div className="bg-white rounded-2xl p-8 shadow-xl w-full max-w-md space-y-6 animate-fade-in">
          <h1 className="text-3xl font-bold text-center text-slate-800">
            Ready to play?
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
              Join matchmaking
            </button>
          </div>
        </div>
      ) : (
        <div className="bg-white rounded-2xl p-8 shadow-xl w-full max-w-lg space-y-6 animate-fade-in">
          <div className="flex justify-between items-center">
            <h2 className="text-2xl font-bold text-slate-800">
              Matchmaking Lobby
            </h2>
            <span className="bg-green-100 text-green-800 px-3 py-1 rounded-full text-sm">
              {connectedPlayers.length + paidPlayers.length} players connected
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
              <p className="text-sm text-slate-500">The game starts in</p>
              <div className="text-4xl font-bold text-blue-600 animate-pulse">
                {countdown}s
              </div>
            </div>
          )}

          <div className="space-y-4">
            <div className="bg-slate-50 p-4 rounded-lg">
              <h3 className="font-medium text-slate-600 mb-2">
                Players waiting for payment
              </h3>
              <div className="space-y-2">
                {connectedPlayers.length === 0 && (
                  <div className="text-xs text-slate-400">No players</div>
                )}
                {connectedPlayers.map((player) => (
                  <div
                    key={player.id}
                    className={`flex items-center p-2 rounded-md ${
                      player.id === wallet
                        ? "bg-blue-100 border border-blue-200"
                        : "bg-white"
                    }`}
                  >
                    <div className="h-2 w-2 rounded-full bg-yellow-400 mr-3" />
                    <span className="font-medium">{player.username}</span>
                    <span className="ml-2 font-mono text-xs text-slate-400">
                      {player.id.slice(0, 8)}...
                    </span>
                    {player.id === wallet && (
                      <span className="ml-2 px-2 py-1 bg-blue-600 text-white text-xs rounded-full">
                        You
                      </span>
                    )}
                  </div>
                ))}
              </div>
            </div>

            <div className="bg-slate-50 p-4 rounded-lg">
              <h3 className="font-medium text-slate-600 mb-2">
                Ready players (paid)
              </h3>
              <div className="space-y-2">
                {paidPlayers.length === 0 && (
                  <div className="text-xs text-slate-400">No ready players</div>
                )}
                {paidPlayers.map((player) => (
                  <div
                    key={player.id}
                    className={`flex items-center p-2 rounded-md ${
                      player.id === wallet
                        ? "bg-green-100 border border-green-200"
                        : "bg-white"
                    }`}
                  >
                    <div className="h-2 w-2 rounded-full bg-green-500 mr-3" />
                    <span className="font-medium">{player.username}</span>
                    <span className="ml-2 font-mono text-xs text-slate-400">
                      {player.id.slice(0, 8)}...
                    </span>
                    {player.id === wallet && (
                      <span className="ml-2 px-2 py-1 bg-green-600 text-white text-xs rounded-full">
                        You
                      </span>
                    )}
                  </div>
                ))}
              </div>
            </div>

            <div className="bg-slate-100 p-3 rounded text-xs text-slate-600">
              {getLobbyStatus()}
            </div>
            <ActionButtons
              isPaid={isPaid}
              isConnected={isConnected}
              countdown={countdown}
              onPay={handlePayClick}
              onCancelPayment={cancelPayment}
              onLeave={disconnect}
            />
          </div>
        </div>
      )}
      <CryptoPaymentModal
        open={showPaymentModal}
        onConfirm={handleConfirmPayment}
        onCancel={handleCancelPayment}
      />
    </div>
  )
}
