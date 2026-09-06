/**
 * @file      gc3plaintextedit.h
 * @brief     A plain text edit that can show a watermark across its viewport.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#ifndef GC3PLAINTEXTEDIT_H
#define GC3PLAINTEXTEDIT_H

#include <QPaintEvent>
#include <QPlainTextEdit>

namespace genc3wb::widget
{

/**
 * @brief   A plain text edit that draws a watermark across its viewport.
 * @details * A null text shows the watermark '- null -' instead of an empty document.
 * @par prefix wdgt
 */
class Gc3PlainTextEdit : public QPlainTextEdit
{
    Q_OBJECT

public: // constructors / destructors
    explicit Gc3PlainTextEdit(QWidget* pwdgtParent = nullptr);
    virtual ~Gc3PlainTextEdit();

public: // methods
    /** @brief Sets the watermark and whether it is drawn. */
    virtual void setWatermark(const QString& strWatermark, bool bEnable = true);

    /** @brief Sets whether the watermark is drawn. */
    virtual void enableWatermark(bool bEnable);

    /** @brief Yields the watermark set. */
    virtual const QString& strWatermark() const;

    /** @brief Yields whether the watermark is drawn. */
    virtual bool isWatermarkEnabled() const;

    /** @brief Shows the document as null, with the watermark, or as an ordinary document. */
    virtual void setNull(bool bNull = true);

    /** @brief Sets the text, showing a null text as null. */
    virtual void setPlainText(const QString& strText);

protected: // overridden functions
    virtual void paintEvent(QPaintEvent* pEvent) override;

protected: // attributes
    QString     m_strWatermark;     ///< the text drawn across the viewport
    bool        m_bEnable;          ///< whether the watermark is drawn
};

}

#endif // GC3PLAINTEXTEDIT_H
