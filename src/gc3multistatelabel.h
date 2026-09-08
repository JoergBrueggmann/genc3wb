/**
 * @file      gc3multistatelabel.h
 * @brief     A label that shows one of several states, each by its own style sheet.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#ifndef GC3MULTISTATELABEL_H
#define GC3MULTISTATELABEL_H

#include <QLabel>
#include <QStringList>

namespace genc3wb::widget
{

/**
 * @brief   A label that shows one of several states, each by its own style sheet.
 * @details * The state is an index into the style sheets given at construction.
 *          * A state outside that range is ignored, and the label keeps the state it had.
 * @par prefix wdgt
 */
class Gc3MultiStateLabel : public QLabel
{
    Q_OBJECT

public: // constructors / destructors
    explicit Gc3MultiStateLabel(const QStringList& strListStyleSheets, QWidget* pwdgtParent = nullptr);
    virtual ~Gc3MultiStateLabel();

public: // methods
    /**
     * @brief   Shows the state of that index.
     * @param   nState  the index of the style sheet to show
     * @return  whether the index was within its range and the state was shown
     */
    virtual bool setState(int nState);

    /**
     * @brief   Yields the state shown.
     * @return  the index of the style sheet shown, -1 where none was shown yet
     */
    virtual int nState() const;

    /**
     * @brief   Yields how many states the label can show.
     * @return  the number of style sheets given at construction
     */
    virtual int nStateCount() const;

protected: // attributes
    QStringList     m_strListStyleSheets;   ///< one style sheet per state
    int             m_nState;               ///< the state shown, -1 where none was shown yet
};

}

#endif // GC3MULTISTATELABEL_H
