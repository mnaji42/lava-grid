"use client"
import { useEffect, useState } from "react"
import Matchmaking from "./matchmaking/page"

export default function Home() {
  // Gestion du wallet mock
  const [wallet, setWallet] = useState<string | null>(null)

  useEffect(() => {
    if (typeof window !== "undefined") {
      setWallet(localStorage.getItem("wallet"))
    }
  }, [])

  // Login = génération d'un wallet mock
  const handleLogin = () => {
    const newWallet = crypto.randomUUID()
    localStorage.setItem("wallet", newWallet)
    setWallet(newWallet)
  }

  // Logout = suppression du wallet et du username
  const handleLogout = () => {
    localStorage.removeItem("wallet")
    localStorage.removeItem("username")
    setWallet(null)
  }

  return (
    <div className="grid grid-rows-[20px_1fr_20px] items-center justify-items-center min-h-screen p-8 pb-20 gap-16 sm:p-20 font-[family-name:var(--font-geist-sans)]">
      <main className="flex flex-col gap-[32px] row-start-2 items-center sm:items-start">
        {!wallet ? (
          <div className="flex flex-col items-center gap-6">
            <h1 className="text-3xl font-bold text-slate-800 mb-2">Welcome!</h1>
            <button
              onClick={handleLogin}
              className="py-3 px-8 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-all text-lg"
            >
              Login
            </button>
            <p className="text-slate-500 text-center max-w-xs">
              Cliquez sur "Login" pour générer un wallet mock et accéder au
              matchmaking.
            </p>
          </div>
        ) : (
          <>
            <div className="flex flex-col items-center gap-2 mb-4">
              <span className="text-xs text-slate-500">Connected as:</span>
              <span className="font-mono bg-slate-100 px-3 py-1 rounded text-slate-800 text-sm">
                {wallet}
              </span>
              <button
                onClick={handleLogout}
                className="mt-2 py-1 px-4 bg-red-600 hover:bg-red-700 text-white rounded text-xs"
              >
                Logout
              </button>
            </div>
            <Matchmaking wallet={wallet} />
          </>
        )}
      </main>
      <footer className="row-start-3 flex gap-[24px] flex-wrap items-center justify-center"></footer>
    </div>
  )
}
