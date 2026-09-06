/**
 * @file      gc3lineedit.h
 * @brief     A line edit that shows whether it is read-only and can suppress its change event.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#ifndef GC3LINEEDIT_H
#define GC3LINEEDIT_H

#include <QLineEdit>

namespace genc3wb::widget
{

/**
 * @brief   A line edit that colours itself according to whether it is read-only.
 * @details * Text set by the program, rather than typed by the user, is marked to be omitted once.
 * @par prefix wdgt
 */
class Gc3LineEdit : public QLineEdit
{
    Q_OBJECT

public: // constructors / destructors
    explicit Gc3LineEdit(QWidget* pwdgtParent = nullptr);
    Gc3LineEdit(const QString& strContents, QWidget* pwdgtParent = nullptr);
    virtual ~Gc3LineEdit();

public: // methods
    /** @brief Sets the read-only state and the colour that shows it. */
    virtual void setReadOnly(bool bReadOnly);

    /** @brief Sets the text and marks the change that follows as one to be omitted. */
    virtual void setText(const QString& strText);

    /**
     * @brief   Yields whether the next change event is to be omitted, and clears the mark.
     * @return  whether the change was caused by the program rather than by the user
     */
    virtual bool isChangeToBeOmitted();

public: // overridden functions
    virtual bool event(QEvent* pEvent) override;

protected: // internal methods
    virtual void updateColor(bool bReadOnly);

protected: // attributes
    bool    m_bOmitChangeEvent;     ///< whether the next change event was caused by the program
};

}

#endif // GC3LINEEDIT_H
