/**
 * @file      gc3timerwatchdog.h
 * @brief     A timer that expires when it has not been kept alive.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#ifndef GC3TIMERWATCHDOG_H
#define GC3TIMERWATCHDOG_H

#include <QTimer>

namespace genc3wb::widget
{

// realises FR-016
/**
 * @brief   A timer that expires when it has not been kept alive for a number of ticks.
 * @details * Every tick without a call of alive() counts towards the threshold.
 *          * On expiry the watchdog emits expired() and starts counting again.
 * @par prefix wdgt
 */
class Gc3TimerWatchdog : public QTimer
{
    Q_OBJECT

public: // constructors / destructors
    /**
     * @brief   Starts the watchdog.
     * @param   nMilliSecondsPerTick    the length of one tick
     * @param   nTicksThreshold         the number of ticks without a call of alive() that makes it expire
     * @param   bInitiallyEnabled       whether it counts from the start
     */
    Gc3TimerWatchdog(int nMilliSecondsPerTick, int nTicksThreshold, bool bInitiallyEnabled = false);
    virtual ~Gc3TimerWatchdog();

public: // methods
    /** @brief Keeps the watchdog alive: enables it and clears the ticks counted. */
    virtual void alive();

    /** @brief Yields whether the watchdog is counting. */
    virtual bool isEnabled() const;

    /** @brief Yields how many ticks were counted without a call of alive(). */
    virtual int nTicksNoKeepAlive() const;

    /** @brief Stops the timer. */
    virtual void suspend();

    /** @brief Starts the timer again. */
    virtual void resume();

signals:
    /** @brief Announces that the watchdog was not kept alive for the threshold of ticks. */
    void expired();

protected slots:
    virtual void on_timerTick();

protected: // attributes
    int     m_nMilliSecondsPerTick;     ///< the length of one tick
    int     m_nTicksNoKeepAlive;        ///< the ticks counted since the last call of alive()
    int     m_nTicksThreshold;          ///< the ticks that make the watchdog expire
    bool    m_bEnabled;                 ///< whether the watchdog is counting
};

}

#endif // GC3TIMERWATCHDOG_H
