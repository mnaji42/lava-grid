"use client"
import { useEffect, useRef, useState } from "react"
import { useRouter, useParams } from "next/navigation"

type CasinoModeSelectionProps = {
  modes: string[]
  players: Player[]
  votes: Record<string, string | null>
  deadline: number // timestamp ms
  onVote: (mode: string) => void
  myPlayerId: string
  timer: number
  hasVoted: boolean
  chosenMode?: string // Optionnel, à afficher à la fin du tirage
  chosenPlayerId?: string // Optionnel, joueur tiré au sort
}

type GameModeSelectionProps = {
  modes: string[]
  players: Player[]
  votes: Record<string, string | null> // playerId -> mode choisi ou null
  deadline: number // timestamp ms
  onVote: (mode: string) => void
  myPlayerId: string
  timer: number
  hasVoted: boolean
}

const GameModeSelectionOverlay: React.FC<GameModeSelectionProps> = ({
  modes,
  votes,
  onVote,
  myPlayerId,
  timer,
  hasVoted,
}) => {
  const votesPerMode: Record<string, number> = {}
  modes.forEach((mode) => {
    votesPerMode[mode] = Object.values(votes).filter((v) => v === mode).length
  })

  return (
    <div className="absolute left-0 top-0 w-full h-full flex items-center justify-center z-30 pointer-events-auto">
      <div className="bg-gray-900/95 border-2 border-blue-700 rounded-xl shadow-2xl px-8 py-6 min-w-[350px] max-w-[90vw] flex flex-col items-center relative">
        <div className="absolute right-4 top-4 text-xs text-slate-400 font-mono">
          <span className="font-bold text-blue-400 text-lg">{timer}s</span>
        </div>
        <h2 className="text-xl font-bold mb-2 text-blue-300 text-center">
          Choisissez le mode de jeu
        </h2>
        <p className="mb-4 text-slate-300 text-center">
          Votez pour le mode de jeu que vous souhaitez jouer.
          <br />
          Le mode avec le plus de votes sera sélectionné.
          <br />
          <span className="text-xs text-slate-400">
            (Vous pouvez voter une seule fois)
          </span>
        </p>
        <div className="flex flex-row gap-4 mb-2">
          {modes.map((mode) => (
            <button
              key={mode}
              className={`
                px-5 py-2 rounded-lg border-2 font-semibold text-lg transition
                ${
                  hasVoted
                    ? votes[myPlayerId] === mode
                      ? "bg-blue-700 border-blue-400 text-white shadow"
                      : "bg-gray-800 border-gray-600 text-slate-400 opacity-60 cursor-not-allowed"
                    : "bg-gray-800 border-gray-600 hover:bg-blue-800 hover:border-blue-400 text-white"
                }
              `}
              disabled={hasVoted}
              onClick={() => onVote(mode)}
            >
              {mode}
              <span className="ml-2 text-xs font-normal text-blue-300">
                ({votesPerMode[mode]})
              </span>
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}

type Player = {
  id: string // changed to string for UUID support
  username: string
  pos?: { x: number; y: number } // optional in pre-game
  cannonball_count?: number
  is_alive?: boolean
}

type Cannonball = {
  pos: { x: number; y: number }
}

type GameState = {
  grid: string[][]
  players: Player[]
  cannonballs: Cannonball[]
  turn: number
  targeted_tiles: any[]
}

type PreGameData = {
  modes: string[]
  deadline_secs: number
  players: Player[]
  grid_row: number
  grid_col: number
}

type GameWsMessage =
  | {
      action: "GameStateUpdate"
      data: { state: GameState; turn_duration: number }
    }
  | { action: "GameEnded"; data: { winner: string } }
  | { action: "GamePreGameData"; data: PreGameData }
  | { action: string; data: any }

export default function GamePage() {
  const [ws, setWs] = useState<WebSocket | null>(null)
  const [gameState, setGameState] = useState<GameState | null>(null)
  const [connectionStatus, setConnectionStatus] = useState("Connecting...")
  const [wallet, setWallet] = useState<string | null>(null)
  const [username, setUsername] = useState<string>("")
  const [mounted, setMounted] = useState(false)
  const [modeVotes, setModeVotes] = useState<Record<string, string | null>>({})
  const [myPlayerId, setMyPlayerId] = useState<string>("")

  // Pre-game state
  const [preGame, setPreGame] = useState<PreGameData | null>(null)
  const [modeVote, setModeVote] = useState<string | null>(null)
  const [voteDeadline, setVoteDeadline] = useState<number>(0)
  const [voteTimer, setVoteTimer] = useState<number>(0)
  const voteTimerRef = useRef<NodeJS.Timeout | null>(null)

  // Animation roulette state
  const [rouletteIndex, setRouletteIndex] = useState(0)
  const [isRolling, setIsRolling] = useState(false)
  const rouletteIntervalRef = useRef<NodeJS.Timeout | null>(null)

  // Timer state
  const [turnTimer, setTurnTimer] = useState<number>(0)
  const [turnDuration, setTurnDuration] = useState<number>(0)
  const [phase, setPhase] = useState<"move" | "wait" | "idle" | "pregame">(
    "idle"
  )
  const [hasPlayed, setHasPlayed] = useState<boolean>(false)
  const timerRef = useRef<NodeJS.Timeout | null>(null)
  const lastTurnRef = useRef<number>(0)

  const { gameId } = useParams()
  const router = useRouter()

  useEffect(() => {
    setMounted(true)
  }, [])

  useEffect(() => {
    if (!mounted) return
    const w = localStorage.getItem("wallet")
    if (!w) {
      router.replace("/")
      return
    }
    setWallet(w)
    setUsername(localStorage.getItem("username") || "")
  }, [mounted, router])

  useEffect(() => {
    if (preGame) {
      // Initialise les votes à null
      const votesInit: Record<string, string | null> = {}
      preGame.players.forEach((p) => {
        votesInit[p.id] = null
      })
      setModeVotes(votesInit)
    }
  }, [preGame])

  useEffect(() => {
    if (preGame && wallet) {
      // Trouve le playerId correspondant à ce wallet/username
      const me = preGame.players.find((p) => p.username === username)
      if (me) setMyPlayerId(me.id)
    }
  }, [preGame, wallet, username])

  // Logout
  const handleLogout = () => {
    localStorage.removeItem("wallet")
    localStorage.removeItem("username")
    setWallet(null)
    router.replace("/")
  }

  // Timer logic: precise countdown with setInterval
  const startTurnTimer = (duration: number) => {
    if (timerRef.current) clearInterval(timerRef.current)
    setTurnTimer(duration)
    let timeLeft = duration
    timerRef.current = setInterval(() => {
      timeLeft -= 1
      setTurnTimer(timeLeft)
      if (timeLeft <= 0) {
        if (timerRef.current) clearInterval(timerRef.current)
        timerRef.current = null
      }
    }, 1000)
  }

  // Pre-game vote timer
  const startVoteTimer = (duration: number) => {
    if (voteTimerRef.current) clearInterval(voteTimerRef.current)
    setVoteTimer(duration)
    let timeLeft = duration
    voteTimerRef.current = setInterval(() => {
      timeLeft -= 1
      setVoteTimer(timeLeft)
      if (timeLeft <= 0) {
        if (voteTimerRef.current) clearInterval(voteTimerRef.current)
        voteTimerRef.current = null
      }
    }, 1000)
  }

  // Clean up timers on unmount
  useEffect(() => {
    return () => {
      if (timerRef.current) clearInterval(timerRef.current)
      if (voteTimerRef.current) clearInterval(voteTimerRef.current)
    }
  }, [])

  // WebSocket
  useEffect(() => {
    if (!wallet || !gameId || !username) return

    const socket = new WebSocket(
      `ws://localhost:8080/ws/game/${gameId}?wallet=${encodeURIComponent(
        wallet
      )}&username=${encodeURIComponent(username)}`
    )

    socket.onopen = () => {
      setConnectionStatus("Connected")
    }

    socket.onmessage = (event) => {
      const msg: GameWsMessage = JSON.parse(event.data)
      console.log(msg)
      switch (msg.action) {
        case "GameModeVoteUpdate": {
          // msg.data: { player_id, mode }
          setModeVotes((prev) => ({
            ...prev,
            [msg.data.player_id]: msg.data.mode,
          }))
          break
        }
        case "GamePreGameData": {
          setPreGame(msg.data)
          setPhase("pregame")
          setModeVote(null)
          setVoteDeadline(Date.now() + msg.data.deadline_secs * 1000)
          startVoteTimer(msg.data.deadline_secs)
          // Initialize a dummy game state for grid display
          const grid = Array.from({ length: msg.data.grid_row }, () =>
            Array.from({ length: msg.data.grid_col }, () => "Solid")
          )
          setGameState({
            grid,
            players: msg.data.players.map((p) => ({
              ...p,
              pos: undefined,
              cannonball_count: 0,
              is_alive: true,
            })),
            cannonballs: [],
            turn: 0,
            targeted_tiles: [],
          })
          break
        }
        case "GameStateUpdate": {
          setPreGame(null)
          setGameState(msg.data.state)
          setTurnDuration(msg.data.turn_duration)
          setPhase("move")
          // Si nouveau tour, reset phase et timer
          if (lastTurnRef.current !== msg.data.state.turn) {
            lastTurnRef.current = msg.data.state.turn
            setHasPlayed(false)
            setPhase("move")
            // Timer = TURN_DURATION - 2s (latence safe)
            const safeDuration = Math.max(1, (msg.data.turn_duration || 5) - 1)
            startTurnTimer(safeDuration)
          }
          break
        }
        case "GameEnded":
          router.push(`/game-over/${msg.data.winner}`)
          break
        default:
          // Optionally handle other actions
          break
      }
    }

    socket.onclose = () => {
      setConnectionStatus("Disconnected")
      if (timerRef.current) clearInterval(timerRef.current)
      if (voteTimerRef.current) clearInterval(voteTimerRef.current)
    }

    setWs(socket)
    return () => {
      socket.close()
      if (timerRef.current) clearInterval(timerRef.current)
      if (voteTimerRef.current) clearInterval(voteTimerRef.current)
    }
  }, [gameId, wallet, username, router])

  // Envoi des mouvements
  const sendMove = (direction: string) => {
    if (
      ws &&
      ws.readyState === WebSocket.OPEN &&
      phase === "move" &&
      !hasPlayed
    ) {
      ws.send(
        JSON.stringify({
          action: "Move",
          data: direction,
        })
      )
      setHasPlayed(true)
      setPhase("wait")
    }
  }

  // Envoi du vote de mode
  const sendModeVote = (mode: string) => {
    if (
      ws &&
      ws.readyState === WebSocket.OPEN &&
      phase === "pregame" &&
      !modeVote
    ) {
      ws.send(
        JSON.stringify({
          action: "GameModeVote",
          data: {
            mode: mode,
          },
        })
      )
      setModeVote(mode)
    }
  }

  const moveGohstPlayer = (key) => {
    const ghost = document.getElementById("ghostPlayer")
    if (!ghost) return
    ghost.style.transition = "transform 0.3s ease"
    switch (key) {
      case "ArrowUp":
        ghost.style.transform = "translateY(-100%)"
        break
      case "ArrowDown":
        ghost.style.transform = "translateY(100%)"
        break
      case "ArrowLeft":
        ghost.style.transform = "translateX(-100%)"
        break
      case "ArrowRight":
        ghost.style.transform = "translateX(100%)"
        break
    }
  }

  const removeGohstPlayer = () => {
    const ghost = document.getElementById("ghostPlayer")
    if (!ghost) return
    ghost.style.transition = "none"
    ghost.style.transform = "none"
  }

  // Gestion des touches clavier
  useEffect(() => {
    if (!ws) return
    const handleKeyPress = (e: KeyboardEvent) => {
      if (phase !== "move" || hasPlayed) return
      switch (e.key) {
        case "ArrowUp":
        case "ArrowDown":
        case "ArrowLeft":
        case "ArrowRight":
          e.preventDefault()
          sendMove(e.key.replace("Arrow", ""))
          moveGohstPlayer(e.key)
          break
        default:
          break
      }
    }
    window.addEventListener("keydown", handleKeyPress)
    return () => window.removeEventListener("keydown", handleKeyPress)
  }, [ws, phase, hasPlayed])

  useEffect(() => {
    if (phase === "pregame" && preGame) {
      setIsRolling(true)
      if (rouletteIntervalRef.current)
        clearInterval(rouletteIntervalRef.current)
      rouletteIntervalRef.current = setInterval(() => {
        setRouletteIndex((prev) => (prev + 1) % preGame.players.length)
      }, 200)
    } else {
      setIsRolling(false)
      if (rouletteIntervalRef.current)
        clearInterval(rouletteIntervalRef.current)
    }
    return () => {
      if (rouletteIntervalRef.current)
        clearInterval(rouletteIntervalRef.current)
    }
  }, [phase, preGame])

  // Sécuriser le rendu : attendre que le composant soit monté et que le wallet soit chargé
  if (!mounted) return null
  if (!wallet) return <div className="p-4 text-center">Redirecting...</div>
  if (!gameState)
    return (
      <div className="p-4 text-center">
        <div className="mb-2">
          <span className="text-xs text-slate-500">Your wallet:</span>
          <span className="font-mono bg-slate-100 px-2 py-1 rounded text-slate-800 text-xs ml-2">
            {wallet}
          </span>
          <button
            onClick={handleLogout}
            className="ml-4 py-1 px-4 bg-red-600 hover:bg-red-700 text-white rounded text-xs"
          >
            Logout
          </button>
        </div>
        {connectionStatus}
      </div>
    )

  // Helper pour afficher les cannonballs
  const renderCannonballs = (count: number = 0) =>
    Array.from({ length: count }).map((_, i) => (
      <span key={i} role="img" aria-label="cannonball" className="ml-0.5">
        💣
      </span>
    ))

  // Helper pour savoir si c'est le joueur courant
  const isCurrentPlayer = (player: Player) => {
    return player.username === username
  }

  const isCurrentPlayerPosition = (x: number, y: number) => {
    const currPlayer = gameState.players.find(
      (player) => player.username === username
    )
    return currPlayer?.pos && currPlayer.pos.x === x && currPlayer.pos.y === y
  }

  // Affichage du timer et du message d'action
  let actionMessage = ""
  if (phase === "move") {
    actionMessage = hasPlayed ? "Waiting for other players..." : "Your turn!"
  } else if (phase === "wait") {
    actionMessage = "Waiting for other players..."
  }

  const getTailwindCell = (
    cell: string,
    currentPlayerHere: boolean,
    cannonballHere: boolean
  ) => {
    let tailwind = ``

    // Handle border:
    if (currentPlayerHere) tailwind += " border-blue-500"
    else if (cannonballHere) tailwind += "  border-yellow-400"
    else if (cell === "Solid") tailwind += " border-gray-800"
    else if (cell === "Broken") tailwind += " border-red-900"

    // Handle bg:
    if (cell === "Solid") tailwind += " bg-gray-700"
    else if (cell === "Broken") tailwind += " bg-red-900"

    return (
      "aspect-square flex items-center justify-center border-2 rounded transition" +
      tailwind
    )
  }

  const getPlayerAt = (x: number, y: number): Player | undefined =>
    gameState.players.find(
      (p) => p.pos && p.pos.x === x && p.pos.y === y && p.is_alive
    )

  // Helper: get cannonball at a given cell
  const isCannonballAt = (x: number, y: number): boolean =>
    gameState.cannonballs.some((c) => c.pos.x === x && c.pos.y === y)

  return (
    <div className="min-h-screen bg-gray-900 text-white p-4">
      <div className="max-w-2xl mx-auto">
        {/* Header d'infos de partie */}
        <div className="flex flex-col md:flex-row md:items-center md:justify-between mb-6 gap-2">
          <div>
            <h1 className="text-2xl font-bold mb-1">Partie #{gameId}</h1>
            <div className="text-xs text-slate-400">
              Status: <span className="font-semibold">{connectionStatus}</span>
            </div>
            <div className="text-xs text-slate-400">
              Tour actuel:{" "}
              <span className="font-semibold text-yellow-300">
                {gameState.turn}
              </span>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs text-slate-400">Wallet:</span>
            <span className="font-mono bg-slate-800 px-2 py-1 rounded text-xs">
              {wallet}
            </span>
            <button
              onClick={handleLogout}
              className="ml-2 py-1 px-4 bg-red-600 hover:bg-red-700 text-white rounded text-xs"
            >
              Logout
            </button>
          </div>
        </div>

        <div className="flex flex-row-reverse gap-2">
          {/* Liste des joueurs */}
          <div className="bg-gray-800 rounded-lg p-3 shadow flex flex-col gap-2 max-w-[30%] relative">
            <div className="font-semibold text-slate-300 mb-1">Joueurs</div>
            <div className="flex flex-col gap-2">
              {gameState.players.map((player, idx) => {
                // Ajout : récupération du vote pour ce joueur
                const votedMode = modeVotes[player.id]
                const isRouletteHighlighted =
                  phase === "pregame" && isRolling && idx === rouletteIndex
                return (
                  <div
                    key={player.id}
                    className={`
                      flex items-center justify-between gap-2 px-3 py-2 rounded transition relative
                      ${
                        player.is_alive
                          ? isCurrentPlayer(player)
                            ? "bg-blue-700/80 border-2 border-blue-400 shadow"
                            : "bg-gray-700/80"
                          : "bg-red-900/80 border border-red-700 opacity-60"
                      }
                      ${
                        isRouletteHighlighted && votedMode
                          ? "ring-4 ring-yellow-300 animate-pulse"
                          : ""
                      }
                    `}
                    style={{
                      transition: "box-shadow 0.2s, border 0.2s",
                    }}
                  >
                    {/* Badge du choix de mode en absolute */}
                    {votedMode && (
                      <span
                        className={`
                          absolute top-1 right-1 px-2 py-0.5 rounded text-xs font-semibold z-10
                          ${
                            votedMode === "Classic"
                              ? "bg-green-700 text-green-200"
                              : "bg-purple-700 text-purple-200"
                          }
                        `}
                        title={`A voté pour ${votedMode}`}
                      >
                        {votedMode}
                      </span>
                    )}
                    <div>
                      <span
                        className={`
                font-bold text-base
                ${player.is_alive ? "" : "line-through text-red-400"}
                ${isCurrentPlayer(player) ? "underline underline-offset-2" : ""}
              `}
                        title={isCurrentPlayer(player) ? "C'est vous !" : ""}
                      >
                        {player.username || `Joueur ${player.id}`}
                      </span>
                      <span className="flex items-center ml-2">
                        {renderCannonballs(player.cannonball_count || 0)}
                      </span>
                    </div>
                    {!player.is_alive && (
                      <div className="ml-2 text-xs text-red-400 font-semibold">
                        💀
                      </div>
                    )}
                  </div>
                )
              })}
            </div>
          </div>
          <div className="relative grow-1">
            {/* Timer et message d'action */}
            <div className="absolute left-[50%] top-[50%] -translate-x-1/2 -translate-y-1/2 w-full flex items-center justify-center mb-4 gap-4">
              <div className="text-lg font-semibold opacity-50">
                {actionMessage}
              </div>
              {phase !== "idle" && (
                <div className="text-2xl font-bold text-blue-400 bg-gray-800 px-4 py-1 rounded shadow animate-pulse opacity-80">
                  {turnTimer}s
                </div>
              )}
            </div>

            {/* Grille de jeu */}
            <div className="bg-gray-800 p-2 rounded-lg text-2xl">
              {phase === "pregame" && preGame && (
                // <GameModeSelectionOverlay
                //   modes={preGame.modes}
                //   players={preGame.players}
                //   votes={modeVotes}
                //   deadline={voteDeadline}
                //   onVote={sendModeVote}
                //   myPlayerId={myPlayerId}
                //   timer={voteTimer}
                //   hasVoted={!!modeVote}
                // />
                <GameModeSelectionOverlay
                  modes={preGame.modes}
                  players={preGame.players}
                  votes={modeVotes}
                  deadline={voteDeadline}
                  onVote={sendModeVote}
                  myPlayerId={myPlayerId}
                  timer={voteTimer}
                  hasVoted={!!modeVote}
                />
              )}
              <div
                className="grid gap-1 border-gray-800 bg-red-900 rounded-lg"
                style={{
                  gridTemplateColumns: `repeat(${gameState.grid[0].length}, minmax(0, 1fr))`,
                }}
              >
                {gameState.grid.map((row, y) =>
                  row.map((cell, x) => {
                    const player = getPlayerAt(x, y)
                    const cannonballHere = isCannonballAt(x, y)
                    const isCurrent = player && player.username === username
                    return (
                      <div
                        key={`${x}-${y}`}
                        className={getTailwindCell(
                          cell,
                          isCurrentPlayerPosition(x, y),
                          cannonballHere
                        )}
                      >
                        {cell === "Broken" && "🔥"}
                        {player && (
                          <div className="relative w-full h-full text-center flex items-center justify-center">
                            <span
                              className={`
                            absolute top-1 left-1/2 -translate-x-1/2
                            px-1 py-0.5 rounded text-xs bg-gray-700/70 text-slate-200
                            pointer-events-none
                            transition
                          `}
                              style={{
                                maxWidth: "90%",
                                whiteSpace: "nowrap",
                                overflow: "hidden",
                                textOverflow: "ellipsis",
                                opacity: 0.85,
                              }}
                              title={player.username}
                            >
                              {player.username}
                            </span>
                            <span role="img" aria-label="player">
                              🧑
                            </span>
                            <>
                              {isCurrent && (
                                <div
                                  id="ghostPlayer"
                                  className="w-full h-full absolute opacity-33 flex items-center justify-center"
                                  role="img"
                                  aria-label="player"
                                >
                                  🧑
                                </div>
                              )}
                            </>
                          </div>
                        )}
                        {cannonballHere && (
                          <span role="img" aria-label="cannonball">
                            💣
                          </span>
                        )}
                      </div>
                    )
                  })
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
