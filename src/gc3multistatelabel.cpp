/**
 * @file      gc3multistatelabel.cpp
 * @brief     A label that shows one of several states, each by its own style sheet.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "gc3multistatelabel.h"

namespace genc3wb::widget
{

Gc3MultiStateLabel::Gc3MultiStateLabel(const QStringList& strListStyleSheets, QWidget* pwdgtParent) :
    QLabel(pwdgtParent)
{
    m_strListStyleSheets = strListStyleSheets;
    m_nState = -1;
}

Gc3MultiStateLabel::~Gc3MultiStateLabel()
{
}

bool Gc3MultiStateLabel::setState(int nState)
{
    bool    bValid = ( (nState >= 0) && (nState < (m_strListStyleSheets.size())) );

    if ( bValid ) {
        setStyleSheet(m_strListStyleSheets.at(nState));
        update();
        m_nState = nState;
    }
    return bValid;
}

int Gc3MultiStateLabel::nState() const
{
    return m_nState;
}

int Gc3MultiStateLabel::nStateCount() const
{
    return int(m_strListStyleSheets.size());
}

}
