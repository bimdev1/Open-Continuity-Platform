package dev.oca

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.net.wifi.WifiManager
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat

class OcaService : Service() {

    private var multicastLock: WifiManager.MulticastLock? = null

    override fun onCreate() {
        super.onCreate()
        Log.d("OcaService", "Service created")
        acquireMulticastLock()
        startForegroundService()
        
        // Start Rust core via JNI
        initRustCore()
    }

    private fun acquireMulticastLock() {
        val wifi = applicationContext.getSystemService(Context.WIFI_SERVICE) as WifiManager
        multicastLock = wifi.createMulticastLock("OcaMulticastLock").apply {
            setReferenceCounted(true)
            acquire()
        }
        Log.d("OcaService", "Multicast lock acquired")
    }

    private fun startForegroundService() {
        val channelId = "oca_daemon_channel"
        val channelName = "OCA Daemon Service"
        val manager = getSystemService(NotificationManager::class.java)
        
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_INT >= android.os.Build.VERSION_CODES.O) {
            manager.createNotificationChannel(NotificationChannel(channelId, channelName, NotificationManager.IMPORTANCE_LOW))
        }

        val notification: Notification = NotificationCompat.Builder(this, channelId)
            .setContentTitle("OCA Platform Active")
            .setContentText("Continuity service running in background")
            .setSmallIcon(android.R.drawable.ic_menu_share)
            .build()

        startForeground(1, notification)
    }

    override fun onDestroy() {
        super.onDestroy()
        multicastLock?.release()
        Log.d("OcaService", "Service destroyed")
    }

    override fun onBind(intent: Intent?): IBinder? = null

    // JNI Native methods
    private external fun initRustCore()
    private external fun sendToPeers(text: String, peerAddr: String)

    companion object {
        init {
            System.loadLibrary("oca_jni")
        }
    }
}
