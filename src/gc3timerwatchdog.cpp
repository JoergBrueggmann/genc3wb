/**
 * @file      gc3timerwatchdog.cpp
 * @brief     A timer that expires when it has not been kept alive.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "gc3timerwatchdog.h"

namespace genc3wb::widget
{

Gc3TimerWatchdog::Gc3TimerWatchdog(int nMilliSecondsPerTick, int nTicksThreshold, bool bInitiallyEnabled)
{
    m_nMilliSecondsPerTick = nMilliSecondsPerTick;
    m_nTicksNoKeepAlive    = 0;
    m_nTicksThreshold      = nTicksThreshold;
    m_bEnabled             = bInitiallyEnabled;
    connect(this, &QTimer::timeout, this, &Gc3TimerWatchdog::on_timerTick);
    start(m_nMilliSecondsPerTick);
}

Gc3TimerWatchdog::~Gc3TimerWatchdog()
{
    stop();
}

void Gc3TimerWatchdog::on_timerTick()
{
    if ( m_bEnabled ) {
        m_nTicksNoKeepAlive++;
        if ( m_nTicksNoKeepAlive > m_nTicksThreshold ) {
            m_bEnabled = false;
            emit expired();
            m_nTicksNoKeepAlive = 0;
        }
    }
}

void Gc3TimerWatchdog::alive()
{
    m_bEnabled = true;
    m_nTicksNoKeepAlive = 0;
}

bool Gc3TimerWatchdog::isEnabled() const
{
    return m_bEnabled;
}

int Gc3TimerWatchdog::nTicksNoKeepAlive() const
{
    return m_nTicksNoKeepAlive;
}

void Gc3TimerWatchdog::suspend()
{
    stop();
}

void Gc3TimerWatchdog::resume()
{
    start(m_nMilliSecondsPerTick);
}

}
